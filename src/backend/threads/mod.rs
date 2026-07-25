use crate::backend::{
    mails::types::MailId,
    threads::{cache::Cache, types::ThreadId},
};
use jmap_client::client::Client;
use std::sync::{Arc, Mutex};

mod cache;
pub mod types;

pub struct ThreadsBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
}

/// Methods which are used in the backend
impl ThreadsBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }

    pub fn get_thread(&self, id: &ThreadId) -> Option<Vec<MailId>> {
        let cache = self.cache.lock().unwrap();
        cache
            .get_thread_mails(id)
            .map(|thread_mails| thread_mails.to_vec())
    }
}
