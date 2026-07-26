pub mod mailbox;
pub mod mails;
pub mod task_manager;
pub mod threads;
pub mod types;

use crate::{
    backend::{
        mailbox::types::MailboxId,
        mails::types::MailData,
        task_manager::{TaskId, TaskManager},
        types::CollapsedMail,
    },
    config::Config,
};
use jmap_client::client::Client;
use std::{rc::Rc, sync::Arc};
use tracing::{debug, error, instrument};

type GetState = String;
type QueryState = String;

pub struct Backend {
    config: Rc<Config>,

    client: Arc<Client>,

    mailboxes: Arc<mailbox::MailboxBackend>,
    mails: Arc<mails::MailsBackend>,
    threads: Arc<threads::ThreadsBackend>,

    task_manager: TaskManager,
}

/// Methods needed for `main.rs`
impl Backend {
    pub async fn new() -> Self {
        let config = Rc::new(Config::load().unwrap());

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
            .map(|client| Arc::new(client))
            .unwrap();

        let session = client.session();
        assert!(
            session
                .capabilities()
                .find(|cap| cap.as_str() == jmap_client::URI::Mail.as_ref())
                .is_some(),
            "Hold up! Your server doesn't seem to support email capabilities?! Eh... That's funny... here are the information of the session: {:#?}",
            session
        );

        Self {
            client: client.clone(),
            mailboxes: Arc::new(mailbox::MailboxBackend::new(client.clone())),
            mails: Arc::new(mails::MailsBackend::new(client.clone())),
            threads: Arc::new(threads::ThreadsBackend::new(client.clone())),
            task_manager: TaskManager::new(),
            config,
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        self.task_manager.has_tasks_running()
    }

    pub async fn finish_next_task(&self) {
        self.task_manager.finish_next_task().await;
    }
}

/// Methods for states.
///
/// Method name convention:
/// - `<object>_get_<bla>`: if it's only trying to fetch the data locally
/// - `<object>_get_or_request_<bla>`: if it's trying to fetch the data locally, otherwise creates a request to the server
/// For combined requests
impl Backend {
    #[instrument(skip(self))]
    pub fn mails_get_or_request_collapsed(&self, id: &MailboxId) -> Option<Vec<CollapsedMail>> {
        let mut collapsed_mails: Vec<CollapsedMail> = Vec::with_capacity(
            self.mailbox_get_data(id)
                .map(|mailbox| mailbox.total_threads)
                .unwrap_or(16),
        );

        match self.mails_get_root_mails(id) {
            Some(root_mails_ids) => {
                for root_mail_id in root_mails_ids {
                    let root_mail = self.mail_get_data(&root_mail_id).expect("Requested");
                    let root_mail_thread =
                        self.thread_get(&root_mail.thread_id).expect("Requested");

                    let thread_has_only_one_mail = root_mail_thread.len() == 1;
                    let entry = if thread_has_only_one_mail {
                        CollapsedMail::SingleMail(root_mail_id)
                    } else {
                        CollapsedMail::CollapsedThread(root_mail.thread_id.clone())
                    };

                    collapsed_mails.push(entry);
                }
            }
            None => {
                let id = id.clone();
                let threads = self.threads.clone();
                let mails = self.mails.clone();
                let client = self.client.clone();

                self.task_manager
                    .spawn(TaskId::QueryRootMails(id.clone()), async move {
                        let mut response = {
                            let mut request = client.build();

                            let mail_query_result = {
                                let query_mail = request
                                    .query_email()
                                    .filter(jmap_client::email::query::Filter::InMailbox {
                                        value: id.clone().0,
                                    })
                                    .sort([jmap_client::email::query::Comparator::received_at()
                                        .ascending()])
                                    .position(0)
                                    .limit(10);
                                query_mail.arguments().collapse_threads(true);
                                query_mail.result_reference()
                            };

                            let mail_get_result = request
                                .get_email()
                                .ids_ref(mail_query_result)
                                .properties(MailData::PROPERTIES)
                                .result_reference(jmap_client::email::Property::ThreadId);

                            request.get_thread().ids_ref(mail_get_result);

                            match request.send().await {
                                Ok(r) => r,
                                Err(err) => {
                                    error!("Couldn't send request to server:\n{err}");
                                    return;
                                }
                            }
                        };

                        let thread_get = response
                            .pop_method_response()
                            .unwrap()
                            .unwrap_get_thread()
                            .unwrap();

                        let mail_get = response
                            .pop_method_response()
                            .unwrap()
                            .unwrap_get_email()
                            .unwrap();

                        threads.handle_get_response(thread_get);
                        mails.handle_get_response(mail_get);

                        debug!("Received collapsed mails of '{:?}'.", id.clone());
                    });

                return None;
            }
        }

        Some(collapsed_mails)
    }
}
