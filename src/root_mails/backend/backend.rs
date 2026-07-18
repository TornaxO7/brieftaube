use crate::{
    root_mails::backend::{MailRenderable, RootMailData},
    utils::MailboxId,
};
use jmap_client::client::Client;
use ratatui::widgets::TableState;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;
use tracing::error;

const INIT_ROOT_MAILS: usize = 10;
const DATA_INITIALISED_MSG: &str = "Is initialised";

pub struct Data {
    pub mails: Vec<RootMailData>,
    pub mails_renderable: Vec<MailRenderable>,
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

                let mails_renderable = mails.iter().map(MailRenderable::from).collect();

                let mut data = data.lock().unwrap();
                *data = Some(Data {
                    mails,
                    mails_renderable,
                    state: get_mail_response.take_state(),
                    table_state: TableState::new(),
                });
            }));
    }
}

// `state` methods
impl RootMailsBackend {
    pub fn select_next_mail(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.table_state.select_next();
        }
    }

    pub fn select_previous_mail(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.table_state.select_previous();
        }
    }
}
