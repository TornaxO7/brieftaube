mod cache;
pub mod types;

use crate::backend::{mailbox::types::MailboxId, mails::types::MailData};
use cache::Cache;
use jmap_client::{client::Client, core::response::EmailGetResponse};
use std::sync::{Arc, Mutex};
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

/// helper functions
impl MailsBackend {
    fn is_initialised(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        !cache.is_empty()
    }
}

/// Methods which communicate with the server
impl MailsBackend {}

/// Methods which interact with `Backend`
impl MailsBackend {
    pub fn get_root_mails(&self, id: &MailboxId) -> Option<Vec<MailId>> {
        let cache = self.cache.lock().unwrap();
        cache
            .get_root_mails(id)
            .map(|root_mails| root_mails.to_vec())
    }

    pub fn get_mail(&self, id: &MailId) -> Option<MailData> {
        let cache = self.cache.lock().unwrap();
        cache.get_mail(id).cloned()
    }

    pub fn handle_response(&self, mut response: EmailGetResponse) {
        let mut cache = self.cache.lock().unwrap();

        for mail in response.take_list() {
            cache.add(MailData::new(mail));
        }
        cache.set_state(response.take_state());
    }
}
