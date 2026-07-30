mod cache;
pub mod types;

use crate::backend::{
    Backend,
    mails::types::{MailData, MailDataRest},
    task_manager::TaskId,
    threads::types::ThreadId,
};
use cache::Cache;
use jmap_client::{client::Client, core::response::EmailGetResponse};
use std::sync::{Arc, Mutex};
use tracing::error;
use types::MailId;

const INIT_ROOT_MAILS: usize = 10;

pub struct MailsBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
}

impl MailsBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }
}

/// local getter and setter
impl MailsBackend {
    pub fn get_data(&self, id: &MailId) -> Option<MailData> {
        let cache = self.cache.lock().unwrap();
        cache.get_mail(id).cloned()
    }

    pub fn get_datas(&self, ids: &[MailId]) -> Option<Vec<MailData>> {
        let cache = self.cache.lock().unwrap();

        ids.iter().map(|id| cache.get_mail(id).cloned()).collect()
    }

    pub fn handle_get_response(&self, mut response: EmailGetResponse) {
        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_state());
        for mail in response.take_list() {
            cache.add(MailData::new(mail));
        }
    }

    pub fn handle_rest_get_response(&self, id: &MailId, mut response: EmailGetResponse) {
        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_state());

        let mails = response.take_list();
        debug_assert!(
            mails.len() == 1,
            "Eh... this function should be only called for _one_ mail. ._."
        );

        cache.set_mail_rest(id, MailDataRest::new(&mails[0]));
    }
}

/// Request methods
impl MailsBackend {
    async fn request_mail(&self, id: MailId) -> Result<(), jmap_client::Error> {
        let response = {
            let mut request = self.client.build();
            request
                .get_email()
                .ids(Some([&id.0]))
                .properties(MailData::PROPERTIES);
            request.send_get_email().await?
        };

        self.handle_get_response(response);
        Ok(())
    }
}

impl Backend {
    pub fn mail_get_data(&self, id: &MailId) -> Option<MailData> {
        self.mails.get_data(id)
    }

    pub fn mail_get_or_request_thread_mails(&self, id: &ThreadId) -> Option<Vec<MailData>> {
        let thread_mail_ids = self
            .threads
            .get(id)
            .expect("Thread has already been requested.");

        self.mails.get_datas(&thread_mail_ids).or_else(|| {
            let client = self.client.clone();
            let mails = self.mails.clone();

            self.task_manager.spawn(TaskId::GetThreadMails, async move {
                let response = {
                    let mut request = client.build();

                    request
                        .get_email()
                        .properties(MailData::PROPERTIES)
                        .ids(Some(thread_mail_ids.iter().map(|id| &id.0)));

                    match request.send_get_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send thread-mails request to server:\n{err}");
                            return;
                        }
                    }
                };

                mails.handle_get_response(response);
            });
            None
        })
    }

    pub fn mail_request_rest(&self, id: &MailId) {
        let mail_is_not_fully_fetched = {
            let mail = self.mails.get_data(id).unwrap();
            mail.rest.is_none()
        };

        if mail_is_not_fully_fetched {
            let id = id.clone();
            let client = self.client.clone();
            let mails = self.mails.clone();

            self.task_manager.spawn(TaskId::FetchMailRest, async move {
                let response = {
                    let mut request = client.build();

                    request
                        .get_email()
                        .ids(Some([&id.0]))
                        .properties(MailDataRest::PROPERTIES)
                        .arguments()
                        .fetch_all_body_values(true);

                    match request.send_get_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!(
                                "Couldn't send request to fetch full mail data to server:\n{err}"
                            );
                            return;
                        }
                    }
                };

                mails.handle_rest_get_response(&id, response);
            });
        }
    }
}
