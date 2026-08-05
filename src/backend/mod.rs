pub mod mailbox;
pub mod mails;
mod store;
pub mod task_manager;
pub mod threads;
pub mod types;

pub use mailbox::types::*;
pub use mails::types::*;
pub use threads::types::*;

use crate::{backend::task_manager::TaskManager, config::Config};
use jmap_client::client::Client;
use std::sync::{Arc, Mutex};
use store::Store;

type GetState = String;
type QueryState = String;

/// Methods for states.
///
/// Method name convention:
/// - `<object>_get_<bla>`: if it's only trying to fetch the data locally
/// - `<object>_get_or_request_<bla>`: if it's trying to fetch the data locally, otherwise creates a request to the server
/// For combined requests
pub struct Backend {
    config: Config,
    client: Arc<Client>,
    store: Arc<Mutex<Store>>,
    task_manager: TaskManager,
}

/// Methods needed for `main.rs`
impl Backend {
    // TODO: Error handling
    pub async fn new() -> Self {
        let config = Config::load().unwrap();

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
            .map(|client| Arc::new(client))
            .unwrap();

        let session = client.session();
        assert!(
            session
                .capabilities()
                .find(|cap| cap.as_str() == jmap_client::URI::Mail.as_ref())
                .is_some(),
            "Hold up! Your server doesn't seem to support email capabilities?! Eh... That's funny... here are the information of the session: {:#?}",
            session
        );

        Self {
            client: client.clone(),
            store: Arc::new(Mutex::new(Store::new())),
            task_manager: TaskManager::new(),
            config,
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        self.task_manager.has_tasks_running()
    }

    pub async fn finish_next_task(&self) {
        self.task_manager.finish_next_task().await;
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
