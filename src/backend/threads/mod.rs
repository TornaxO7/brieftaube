use crate::backend::threads::cache::Cache;
use jmap_client::client::Client;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::task::{JoinError, JoinHandle};

mod cache;
pub mod types;

pub struct ThreadsBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    tasks: Mutex<VecDeque<JoinHandle<()>>>,
}

impl ThreadsBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
            tasks: Mutex::new(VecDeque::with_capacity(16)),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<(), JoinError>> {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.front_mut().unwrap();
        Some(task.await)
    }

    pub fn pop_task(&self) {
        self.tasks
            .lock()
            .unwrap()
            .pop_front()
            .expect("There are tasks.");
    }

    pub fn cache_is_initialised(&self) -> bool {
        !self.cache.lock().unwrap().is_empty()
    }
}
