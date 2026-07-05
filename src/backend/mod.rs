use jmap_client::client::Client;

pub struct Account {
    /// Email address
    pub client: jmap_client::client::Client,

    pub address: String,
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
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}
