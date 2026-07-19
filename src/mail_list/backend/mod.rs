mod data_collection;
mod manager;

pub use manager::MailListManager;
pub use data_collection::{DataCollection, types::*, Data, Row, error::UnfoldRowError};

use crate::utils::{EmailKeyword, MailId, MailboxId};
use jmap_client::client::Client;
use std::{
    collections::{ HashSet, VecDeque},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

const DATA_INITIALISED_MSG: &str = "Is initialised";
const INIT_ROOT_MAILS: usize = 10;

pub struct MailListBackend {
    id: MailboxId,
    client: Arc<Client>,
    pub data: Arc<Mutex<Option<DataCollection>>>,
    tasks: Mutex<VecDeque<JoinHandle<()>>>,
}

impl MailListBackend {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new(client: Arc<Client>, id: MailboxId) -> Self {
        Self {
            id,
            client,
            data: Arc::new(Mutex::new(None)),
            tasks: Mutex::new(VecDeque::with_capacity(8)),
        }
    }

    pub fn is_initialised(&self) -> bool {
        self.data.lock().unwrap().is_some()
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
impl MailListBackend {
    pub fn init(&self) {
        if self.is_initialised() {
            // TODO: fetch changes
            return;
        }

        let client = self.client.clone();
        let data = self.data.clone();
        let id = self.id.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let mut request = client.build();

                    let query_result = {
                        let query = request
                            .query_email()
                            .filter(jmap_client::email::query::Filter::InMailbox { value: id })
                            .sort([
                                jmap_client::email::query::Comparator::received_at().ascending()
                            ])
                            .limit(INIT_ROOT_MAILS);
                        query.arguments().collapse_threads(true);
                        query.result_reference()
                    };

                    request.get_email().ids_ref(query_result).properties(MailData::PROPERTIES);

                    match request.send().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request to fetch root mails:\n{err}");
                            return;
                        }
                    }
                };

                let Some(get_mail_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Email/get` method from request.");
                    return;
                };

                let Some(query_mail_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Email/query` method from request.");
                    return;
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

                let mut data = data.lock().unwrap();
                *data = Some(DataCollection::new(query_mail_response, get_mail_response));
            }));
    }

    pub fn update_keywords(&self, ids: Vec<MailId>, keywords: HashSet<EmailKeyword>, add_keywords: bool) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let filtered_ids: Vec<MailId> = {
                    let guard = data.lock().unwrap();
                    let data = guard.as_ref().expect(DATA_INITIALISED_MSG);
                    
                    ids
                        .into_iter()
                        .filter(|id| {
                            data.get_mail(id).map(|mail| {
                                if add_keywords {
                                    mail.keywords.symmetric_difference(&keywords).next().is_some()
                                } else {
                                    mail.keywords.intersection(&keywords).next().is_some()
                                }
                            }).unwrap_or(false)
                        }).collect()
                };

                debug!("{:?} will be applied to: {:?}", &keywords, &filtered_ids);

                if filtered_ids.is_empty() {
                    return;
                }

                let current_state = {
                    let guard = data.lock().unwrap();
                    let data = guard.as_ref().expect(DATA_INITIALISED_MSG);
                    data.get_mail_state()
                };

                let mut response = {
                    let mut request = client.build();
                    let set_mail = request.set_email().if_in_state(current_state);

                    for id in filtered_ids.iter() {
                        for keyword in keywords.iter() {
                            set_mail.update(id).keyword(keyword.to_string().as_str(), add_keywords);
                        }
                    }

                    match request.send_set_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request to server to add keywords: {err}");
                            return;
                        }
                    }
                };

                let mut guard = data.lock().unwrap();
                let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
                data.set_mail_state(response.take_new_state());

                for id in filtered_ids {
                    match response.updated(&id) {
                        Ok(server) => {
                            if server.is_some() {
                                warn!("This shouldn't happen <.<. Please create an issue!");
                            }

                            let Some(mail) = data.get_mail_mut(&id) else {
                                warn!("A mail seems to be removed during the request. Skipping the update of its keywords.");
                                continue;
                            };

                            if add_keywords {
                                mail.keywords.extend(keywords.iter().cloned());
                            } else {
                                mail.keywords = mail.keywords.difference(&keywords).cloned().collect();
                            }
                        }
                        Err(err) => {
                            error!("Couldn't set the keyword for an email: {err}");
                        }
                    }
                }
            }));
    }

    pub fn unfold_thread(&self, row: usize) {
        if !self.is_initialised() {
            return;
        };

        let unfold_row = {
            let mut guard = self.data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.unfold_row(row)
        };

        let thread_id= match unfold_row {
            Ok(()) => return,
            Err(UnfoldRowError::NonRootRow) => {
                warn!("Can't unfold row at index {row}. It's not a root mail in the thread");
                return;
            }
            Err(UnfoldRowError::AlreadyUnfolded) => {
                warn!("Can't unfold thread: Thread is already unfolded.");
                return;
            }
            Err(UnfoldRowError::NotInitialised(thread_id)) => thread_id,
        };

        let data = self.data.clone();
        let client = self.client.clone();
        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let mut response = {
                let mut request = client.build();
                let get_thread_result = {
                    let get = request.get_thread().ids(Some([&thread_id]));
                    get.result_reference(jmap_client::thread::Property::EmailIds)
                };

                request.get_email().ids_ref(get_thread_result).properties(MailData::PROPERTIES);

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
                error!("Couln't pop `Thread/get` method from request.");
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

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.insert_thread(&thread_id, get_mail_response, get_thread_response);
            data.unfold_row(row).expect("Thread just inserted.");
        }));
    }
}

// `state` methods
impl MailListBackend {
    pub fn navigate_to_next_mail(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.table_state.select_next();
        }
    }

    pub fn navigate_to_previous_mail(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.table_state.select_previous();
        }
    }

    pub fn navigate_to_top(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.table_state.select_first();
        }
    }

    pub fn navigate_to_bottom(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.table_state.select_last();
        }
    }

    pub fn get_selected_mail(&self) -> Option<MailData> {
        let guard = self.data.lock().unwrap();
        guard.as_ref().and_then(|data| data.get_selected_mail())
    }

    pub fn get_selected_mail_position(&self) -> Option<usize> {
        let guard = self.data.lock().unwrap();
        guard.as_ref().and_then(|data| data.table_state.selected())
    }
  
    pub fn fold_thread(&self, row: usize) {
        let mut guard = self.data.lock().unwrap();
        guard.as_mut().map(|data| data.fold_row(row));
    }
}