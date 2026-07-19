use crate::{
    root_mails::backend::RootMailData,
    utils::{EmailKeyword, MailId, MailboxId},
};
use jmap_client::client::Client;
use ratatui::widgets::TableState;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;
use tracing::{error, warn};

const INIT_ROOT_MAILS: usize = 10;
const DATA_INITIALISED_MSG: &str = "Is initialised";

pub struct Data {
    pub mails: Vec<RootMailData>,
    // pub mails_renderable: Vec<MailRenderable>,
    mapping: HashMap<MailId, usize>,
    state: String,

    pub table_state: TableState,
}

pub struct RootMailsBackend {
    id: MailboxId,
    client: Arc<Client>,
    pub data: Arc<Mutex<Option<Data>>>,
    tasks: Mutex<VecDeque<JoinHandle<()>>>,
}

impl RootMailsBackend {
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
impl RootMailsBackend {
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
                                jmap_client::email::query::Comparator::received_at().descending()
                            ])
                            .limit(INIT_ROOT_MAILS);
                        query.arguments().collapse_threads(true);
                        query.result_reference()
                    };

                    request.get_email().ids_ref(query_result).properties([
                        jmap_client::email::Property::Id,
                        jmap_client::email::Property::ThreadId,
                        jmap_client::email::Property::Keywords,
                        jmap_client::email::Property::From,
                        jmap_client::email::Property::To,
                        jmap_client::email::Property::Cc,
                        jmap_client::email::Property::Subject,
                        jmap_client::email::Property::Preview,
                        jmap_client::email::Property::ReceivedAt,
                        jmap_client::email::Property::HasAttachment,
                    ]);

                    match request.send().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request to fetch root mails:\n{err}");
                            return;
                        }
                    }
                };

                let Some(get_mail_method) = response.pop_method_response() else {
                    error!("Couldn't pop `Email/get` method from request. :(");
                    return;
                };

                let mut get_mail_response = match get_mail_method.unwrap_get_email() {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't get response of `Email/get` request:\n{err}");
                        return;
                    }
                };

                let mails: Vec<RootMailData> = get_mail_response
                    .take_list()
                    .into_iter()
                    .map(RootMailData::from)
                    .collect();

                let mapping: HashMap<MailId, usize> = mails
                    .iter()
                    .enumerate()
                    .map(|(idx, mail)| (mail.id.clone(), idx))
                    .collect();

                let mut data = data.lock().unwrap();
                *data = Some(Data {
                    mails,
                    mapping,
                    state: get_mail_response.take_state(),
                    table_state: TableState::new(),
                });
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
                    
                    ids.into_iter()
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

                tracing::debug!("{:?}", filtered_ids);

                let current_state = {
                    let guard = data.lock().unwrap();
                    let data = guard.as_ref().expect(DATA_INITIALISED_MSG);
                    data.state.clone()
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
                data.state = response.take_new_state();
                for id in filtered_ids {
                    match response.updated(&id) {
                        Ok(server) => {
                            if server.is_some() {
                                warn!("This shouldn't happen <.<");
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
}

// `state` methods
impl RootMailsBackend {
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

    pub fn get_selected_mail(&self) -> Option<MailId> {
        let guard = self.data.lock().unwrap();
        guard.as_ref().and_then(|data| {
            data.table_state
                .selected()
                .map(|idx| data.mails[idx].id.clone())
        })
    }
}

// helper functions
impl Data {
    fn get_mail(&self, id: &MailId) -> Option<&RootMailData> {
        self.mapping
            .get(id)
            .cloned()
            .and_then(|idx| self.mails.get(idx))
    }

    fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut RootMailData> {
        self.mapping
            .get(id)
            .cloned()
            .and_then(|idx| self.mails.get_mut(idx))
    }
}
