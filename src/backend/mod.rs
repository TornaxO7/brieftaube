use jmap_client::client::Client;

pub struct Account {
    /// Email address
    pub client: jmap_client::client::Client,
    address: String,
}

impl Account {
    pub async fn new() -> Self {
        let config = crate::config::Config::load().unwrap();
        let address = config.address.clone();

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
            .unwrap();

        Self { client, address }
    }

    pub fn address(&self) -> &str {
        self.address.as_str()
    }
}
