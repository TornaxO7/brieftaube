mod mailboxes;

use jmap_client::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Fetcher {
    /// Email address
    client: Arc<jmap_client::client::Client>,
    address: String,

    data: Arc<Mutex<Data>>,
}

impl Fetcher {
    pub async fn new() -> Self {
        let config = crate::config::Config::load().unwrap();

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
            .unwrap();

        let address = config.address.clone();

        Self {
            client: Arc::new(client),
            address,
            data: Arc::new(Mutex::new(Data::default())),
        }
    }

    pub fn refresh(&self) {
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

    pub fn address(&self) -> &str {
        self.address.as_str()
    }
}

#[derive(Default)]
struct Data {
    mailboxes: Option<mailboxes::Mailboxes>,
}
