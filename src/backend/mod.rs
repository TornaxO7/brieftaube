// pub mod mailbox;
// pub mod mails;
// mod store;
// pub mod threads;
// pub mod types;

// pub use mailbox::types::*;
// pub use mails::types::*;
// pub use threads::types::*;
use tokio::sync::watch;

use crate::CONFIG;
use jmap_client::client::Client;
use std::sync::{Arc, Mutex};
// use store::Store;

type GetState = String;
type QueryState = String;

pub enum LoadingRole {
    Wait(watch::Receiver<()>),
    Request(watch::Sender<()>),
}

/// Methods for states.
///
/// Method name convention:
/// - `<object>_get_<bla>`: if it's only trying to fetch the data locally
/// - `<object>_get_or_request_<bla>`: if it's trying to fetch the data locally, otherwise creates a request to the server
/// For combined requests
pub struct Backend {
    client: Arc<Client>,
    // store: Arc<Mutex<Store>>,
}

/// Methods needed for `main.rs`
impl Backend {
    // TODO: Error handling
    pub async fn new() -> Self {
        let config = CONFIG.get().unwrap();

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
            // store: Arc::new(Mutex::new(Store::new())),
        }
    }
}
