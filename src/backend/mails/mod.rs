mod cache;
pub mod types;

use crate::backend::{
    Backend, mails::types::MailData, task_manager::TaskId, threads::types::ThreadId,
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
}
