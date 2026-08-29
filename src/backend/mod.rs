use tokio::sync::{oneshot, watch};

use crate::CONFIG;
use jmap_client::client::Client;
use std::sync::{Arc, Mutex};
// use store::Store;

/// Methods for states.
///
/// Method name convention:
/// - `<object>_get_<bla>`: if it's only trying to fetch the data locally
/// - `<object>_get_or_request_<bla>`: if it's trying to fetch the data locally, otherwise creates a request to the server
/// For combined requests
pub struct Backend {}

/// Methods needed for `main.rs`
impl Backend {
    pub async fn run() {
        // let config = CONFIG.get().unwrap();

        // let client = Client::new()
        //     .credentials((config.address.trim(), config.password.trim()))
        //     .follow_redirects([config.host.trim()])
        //     .connect(&format!("http://{}", config.host.trim()))
        //     .await
        //     .map(|client| Arc::new(client))
        //     .unwrap();

        // let session = client.session();
        // assert!(
        //     session
        //         .capabilities()
        //         .find(|cap| cap.as_str() == jmap_client::URI::Mail.as_ref())
        //         .is_some(),
        //     "Hold up! Your server doesn't seem to support email capabilities?! Eh... That's funny... here are the information of the session: {:#?}",
        //     session
        // );
    }
}
