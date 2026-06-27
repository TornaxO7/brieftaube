use jmap_client::client::Client;

pub struct Account {
    /// Email address
    pub address: String,
    pub client: jmap_client::client::Client,
}

impl Account {
    pub async fn new() -> Self {
        // TODO: Make it configureable
        let home = std::env::var("HOME").unwrap();
        let address = std::fs::read_to_string(format!("{}/stalwart-account.txt", home)).unwrap();
        let client = {
            let host = std::fs::read_to_string(format!("{}/stalwart-host.txt", home)).unwrap();
            let password =
                std::fs::read_to_string(format!("{}/stalwart-password.txt", home)).unwrap();

            let url = format!("http://{}", host.trim());

            Client::new()
                .credentials((address.as_str(), password.trim()))
                .follow_redirects([host.trim()])
                .connect(url.trim())
                .await
                .unwrap()
        };

        Self { client, address }
    }
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}
