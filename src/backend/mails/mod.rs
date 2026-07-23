mod cache;
pub mod types;

use crate::backend::{
    mailbox::types::MailboxId,
    mails::{
        cache::error::UnfoldError,
        types::{MailData, MailDataRest, MailEntry, MailUpdate},
    },
};
use cache::Cache;
use jmap_client::client::Client;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::{debug, error, warn};
use types::{MailId, ThreadId};

const INIT_ROOT_MAILS: usize = 10;

pub struct MailsBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    tasks: Mutex<VecDeque<JoinHandle<()>>>,
}

impl MailsBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::default())),
            tasks: Mutex::new(VecDeque::with_capacity(8)),
        }
    }

    pub fn is_mailbox_initialised(&self, id: &MailboxId) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_mailbox_initialised(id)
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.front_mut().unwrap();
        Some(task.await);
    }

    pub fn pop_task(&self) {
        self.tasks.lock().unwrap().pop_front();
    }
}

// methods which need to interact with the server
impl MailsBackend {
    pub fn init(&self, id: MailboxId) {
        if self.is_mailbox_initialised(&id) {
            // TODO: fetch changes
            return;
        }

        let client = self.client.clone();
        let cache = self.cache.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let mut request = client.build();

                    let query_result = {
                        let query =
                            request
                                .query_email()
                                .filter(jmap_client::email::query::Filter::InMailbox {
                                    value: id.clone(),
                                })
                                .sort([jmap_client::email::query::Comparator::received_at()
                                    .ascending()])
                                .limit(INIT_ROOT_MAILS);
                        query.arguments().collapse_threads(true);
                        query.result_reference()
                    };

                    let get_mail_result = request
                        .get_email()
                        .ids_ref(query_result)
                        .properties(MailData::PROPERTIES)
                        .result_reference(jmap_client::email::Property::ThreadId);

                    let _get_thread = request.get_thread().ids_ref(get_mail_result);

                    match request.send().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request to fetch root mails:\n{err}");
                            return;
                        }
                    }
                };

                let Some(get_thread_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Thread/get` method from request.");
                    return;
                };

                let Some(get_mail_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Email/get` method from request.");
                    return;
                };

                let Some(query_mail_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Email/query` method from request.");
                    return;
                };

                let get_thread_response = match get_thread_method.unwrap_get_thread() {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't get response of `Thread/get` request:\n{err}");
                        return;
                    }
                };

                let get_mail_response = match get_mail_method.unwrap_get_email() {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't get response of `Email/get` request:\n{err}");
                        return;
                    }
                };

                let query_mail_response = match query_mail_method.unwrap_query_email() {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't get response of `Email/query` request:\n{err}");
                        return;
                    }
                };

                let mut cache = cache.lock().unwrap();
                *cache = Cache::init_mailbox(
                    query_mail_response,
                    get_mail_response,
                    get_thread_response,
                );
            }));
    }

    pub fn request_get_mails(&self, mails: Vec<MailId>) {
        let cache = self.cache.clone();
        let client = self.client.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let mut request = client.build();
                    request
                        .get_email()
                        .ids(Some(mails))
                        .properties(MailData::PROPERTIES);
                    match request.send_get_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request mails: {err}");
                            return;
                        }
                    }
                };

                let mut cache = cache.lock().unwrap();
                cache.set_mail_state(response.take_state());

                for mail in response.take_list() {
                    cache.add_mail(MailData::new(mail));
                }
            }));
    }

    pub fn request_get_mails_rest(&self, mails: Vec<MailId>) {
        let cache = self.cache.clone();
        let client = self.client.clone();

        let non_full_mails = {
            let mut ids: Vec<MailId> = Vec::new();

            let cache = self.cache.lock().unwrap();

            for id in mails.into_iter() {
                let mail = cache.get_mail(&id).unwrap();
                if mail.rest.is_none() {
                    ids.push(id);
                }
            }

            ids
        };

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let mut request = client.build();
                    request
                        .get_email()
                        .ids(Some(non_full_mails))
                        .properties(MailDataRest::PROPERTIES)
                        .arguments()
                        .fetch_all_body_values(true);
                    match request.send_get_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request for mail rest to server:\n{err}");
                            return;
                        }
                    }
                };

                let mut cache = cache.lock().unwrap();
                cache.set_mail_state(response.take_state());

                for mut rest_mail in response.take_list() {
                    let mail = cache
                        .get_mail_mut(&rest_mail.take_id())
                        .expect("Response contains mail ids.");

                    mail.extend(&rest_mail);
                }
            }));
    }

    pub fn request_update_mails(&self, mails: Vec<MailUpdate>) {
        let cache = self.cache.clone();
        let client = self.client.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let current_state = {
                        let cache = cache.lock().unwrap();
                        cache.get_mail_state()
                    };

                    let mut request = client.build();
                    let set_mail = request.set_email().if_in_state(current_state);

                    for mail in mails.iter() {
                        let u = set_mail.update(mail.id.clone());

                        if let Some(patches) = &mail.patch_keywords {
                            for (keyword, set) in patches {
                                u.keyword(&keyword.to_string(), *set);
                            }
                        }

                        if let Some(mailbox_ids) = &mail.mailbox_ids {
                            for (id, set) in mailbox_ids {
                                u.mailbox_id(id, *set);
                            }
                        }
                    }

                    match request.send_set_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request server to update the mailboxes:\n{err}");
                            return;
                        }
                    }
                };

                let mut cache = cache.lock().unwrap();
                cache.set_mail_state(response.take_new_state());

                for mail in mails {
                    match response.updated(&mail.id) {
                        Ok(server) => {
                            if let Some(server) = server {
                                debug!("Server send mail back: {server:#?}");
                            }

                            cache.update_mail(mail);
                        }
                        Err(err) => {
                            error!("Couldn't update a mail:\n{err}");
                        }
                    }
                }
            }));
    }

    pub fn request_get_thread_mails(&self, thread_id: &ThreadId) -> oneshot::Receiver<()> {
        let (done_sender, done_receiver) = tokio::sync::oneshot::channel();

        let client = self.client.clone();
        let cache = self.cache.clone();
        let thread_id = thread_id.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let mut request = client.build();
                    let thread_result = request
                        .get_thread()
                        .ids(Some([&thread_id]))
                        .result_reference(jmap_client::thread::Property::EmailIds);

                    request
                        .get_email()
                        .ids_ref(thread_result)
                        .properties(MailData::PROPERTIES);

                    match request.send().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request to fetch mails of thread:\n{err}");
                            return;
                        }
                    }
                };

                let Some(get_mail_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Email/get` method from request.");
                    return;
                };

                let Some(get_thread_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Thread/get` method from request.");
                    return;
                };

                let get_mail_response = match get_mail_method.unwrap_get_email() {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't get response of `Email/get` request:\n{err}");
                        return;
                    }
                };

                let get_thread_response = match get_thread_method.unwrap_get_thread() {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't get response of `Thread/get` request:\n{err}");
                        return;
                    }
                };

                let mut cache = cache.lock().unwrap();
                cache.insert_thread(get_thread_response, get_mail_response);

                if let Err(_err) = done_sender.send(()) {
                    warn!("Couldn't notify other task about finished task.");
                }
            }));

        done_receiver
    }
}

// `state` methods
impl MailsBackend {
    pub fn get_mails(&self, id: &MailboxId) -> Option<Vec<MailEntry>> {
        let cache = self.cache.lock().unwrap();
        cache.get_mails_from_mailbox(id).map(|mails| mails.to_vec())
    }

    pub fn get_mail(&self, id: &MailId) -> Option<MailData> {
        let cache = self.cache.lock().unwrap();
        cache.get_mail(id).cloned()
    }

    /// Returns `true` if there are more than 1 mail in the thread of the mail.
    pub fn has_thread(&self, id: &MailId) -> bool {
        let cache = self.cache.lock().unwrap();

        cache
            .get_mail(id)
            .and_then(|mail| cache.get_thread(&mail.thread_id))
            .map(|thread_mails| thread_mails.len() > 1)
            .unwrap_or(false)
    }

    pub fn fold_thread(&self, mailbox: &MailboxId, thread: &ThreadId) {
        let cache = self.cache.clone();

        let mailbox = mailbox.clone();
        let thread = thread.clone();
        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut cache = cache.lock().unwrap();
                cache.fold_thread(&mailbox, &thread);
            }));
    }

    // TODO: Use an enum instead of a boolean...
    // returns `true` if unfolding was successfull, otherwise `false`
    pub fn unfold_mail(&self, mailbox: &MailboxId, mail: &MailId) -> bool {
        let result = {
            let mut cache = self.cache.lock().unwrap();

            cache.unfold_mail(mailbox, mail)
        };

        match result {
            Ok(just_unfolded) => return just_unfolded,
            Err(UnfoldError::MailboxMailsMissing) => {
                unreachable!("Mails should be already fetched...");
            }
            Err(UnfoldError::MissingThreadMails(thread_id)) => {
                tracing::debug!("request");
                let done = self.request_get_thread_mails(&thread_id);

                let mailbox = mailbox.clone();
                let mail = mail.clone();
                let cache = self.cache.clone();
                self.tasks.lock().unwrap().push_back(tokio::spawn(async move{
                    if let Err(err) = done.await {
                        warn!("Unfolding mail: Can't receive notification after fetching the thread mails anymore :(\n\
                            Can't automatically unfold thread anymore:\n{err}"
                        );
                        return;
                    }

                    let mut cache = cache.lock().unwrap();
                    cache.unfold_mail(&mailbox, &mail).expect("Thread mails should have arrived >:(");
                }));
            }
        };

        false
    }
}
