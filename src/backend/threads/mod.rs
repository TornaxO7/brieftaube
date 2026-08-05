use crate::backend::{
    Backend,
    mails::types::MailId,
    threads::{cache::Cache, types::ThreadId},
};
use jmap_client::core::response::ThreadGetResponse;
use std::sync::{Arc, Mutex};
use tracing::instrument;

mod cache;
pub mod types;

pub struct ThreadsBackend {
    cache: Arc<Mutex<Cache>>,
}

/// Methods which are used in the backend
impl ThreadsBackend {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }

    pub fn get(&self, id: &ThreadId) -> Option<Vec<MailId>> {
        let cache = self.cache.lock().unwrap();
        cache
            .get_thread_mails(id)
            .map(|thread_mails| thread_mails.to_vec())
    }

    #[instrument(skip(self))]
    pub fn handle_get_response(&self, mut response: ThreadGetResponse) {
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

/// Methods which also communicate with the server
impl ThreadsBackend {
    async fn request_get(&self, id: &ThreadId) -> Result<(), jmap_client::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.get_thread().ids(Some([&id.0]));
            request.send_get_thread().await?
        };

        let mut cache = self.cache.lock().unwrap();
        for thread in response.take_list() {
            let thread_id = ThreadId(thread.id().to_owned());
            let mail_ids = thread
                .email_ids()
                .into_iter()
                .cloned()
                .map(|id| MailId(id))
                .collect();

            cache.insert(thread_id, mail_ids);
        }
        cache.set_state(response.take_state());

        Ok(())
    }
}

impl Backend {
    // pub fn threads_get_or_request(&self, id: ThreadId) -> Option<Vec<MailId>> {
    //     let threads = self.threads.clone();

    //     self.thread_get(&id).or_else(|| {
    //         self.task_manager
    //             .spawn(TaskId::ThreadGet(id.clone()), async move {
    //                 match threads.request_get(&id).await {
    //                     Ok(()) => debug!("Retrieved thread ids"),
    //                     Err(err) => error!("Couldn't request mails of thread '{id:?}':\n{err}"),
    //                 }
    //             });
    //         None
    //     })
    // }
}
