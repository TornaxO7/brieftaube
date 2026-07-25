mod cache;
pub mod types;

use crate::backend::{
    Backend, mailbox::types::MailboxId, mails::types::MailData, task_manager::TaskId,
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

// /// helper functions
// impl MailsBackend {
//     fn is_initialised(&self) -> bool {
//         let cache = self.cache.lock().unwrap();
//         !cache.is_empty()
//     }
// }

/// local getter and setter
impl MailsBackend {
    fn get_root_mails(&self, id: &MailboxId) -> Option<Vec<MailId>> {
        let cache = self.cache.lock().unwrap();
        cache
            .get_root_mails(id)
            .map(|root_mails| root_mails.to_vec())
    }

    fn get_mail_data(&self, id: &MailId) -> Option<MailData> {
        let cache = self.cache.lock().unwrap();
        cache.get_mail(id).cloned()
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
    pub fn mails_get_root_mails(&self, id: &MailboxId) -> Option<Vec<MailId>> {
        self.mails.get_root_mails(id)
    }

    pub fn mail_get_data(&self, id: &MailId) -> Option<MailData> {
        self.mails.get_mail_data(id)
    }

    pub fn mail_get_or_request_data(&self, id: MailId) -> Option<MailData> {
        let mails = self.mails.clone();

        self.mail_get_data(&id).or_else(|| {
            self.task_manager
                .spawn(TaskId::MailGet(id.clone()), async move {
                    if let Err(err) = mails.request_mail(id.clone()).await {
                        error!("Couldn't get email:\n{err}");
                    }
                });
            None
        })
    }
}
