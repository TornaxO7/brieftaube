use crate::backend::{
    mails::types::MailId,
    threads::{cache::Cache, types::ThreadId},
};
use jmap_client::{client::Client, core::response::ThreadGetResponse};
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

    pub fn handle_response(&self, mut response: ThreadGetResponse) {
        let mut cache = self.cache.lock().unwrap();

        for thread in response.take_list() {
            let id = ThreadId(thread.id().to_owned());
            let mail_ids = thread
                .email_ids()
                .into_iter()
                .map(|id| MailId(id.clone()))
                .collect();

            cache.insert(id, mail_ids);
        }

        cache.set_state(response.take_state());
    }
}
