mod cache;
pub mod types;

use crate::backend::{mailbox::types::MailboxId, mails::types::MailData};
use cache::Cache;
use jmap_client::client::Client;
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

    pub fn is_initialised(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        !cache.is_empty()
    }
}

// methods which need to interact with the server
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
}
