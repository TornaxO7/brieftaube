mod mailboxes;

use jmap_client::client::Client;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Data {
    mailboxes: Option<mailboxes::Mailboxes>,
}

pub struct Account {
    client: Arc<jmap_client::client::Client>,
    data: Arc<Mutex<Data>>,
}

impl Account {
    pub async fn new() -> Self {
        let config = crate::config::Config::load().unwrap();

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
            .unwrap();

        Self {
            client: Arc::new(client),
            data: Arc::new(Mutex::new(Data::default())),
        }
    }

    pub async fn fetch_changes(&self) {
        let mut data = self.data.lock().unwrap();
        let client = self.client.clone();

        match data.mailboxes.as_mut() {
            Some(data) => data.fetch_changes(&client).await,
            None => data.mailboxes = Some(mailboxes::Mailboxes::new(&client).await),
        };
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }
}
