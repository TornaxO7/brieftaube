mod mailboxes;

use jmap_client::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
struct FetcherData {
    mailboxes: Option<mailboxes::Mailboxes>,
}

pub struct Account {
    pub client: Arc<jmap_client::client::Client>,
    data: Arc<Mutex<FetcherData>>,
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
            data: Arc::new(Mutex::new(FetcherData::default())),
        }
    }

    pub fn fetch_changes(&self) {
        let data = self.data.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut data = data.lock().await;

            match data.mailboxes.as_mut() {
                Some(data) => data.fetch_changes(&client).await,
                None => data.mailboxes = Some(mailboxes::Mailboxes::new(&client).await),
            };
        });
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }
}
