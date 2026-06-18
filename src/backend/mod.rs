use jmap_client::mailbox::query::Filter;

pub struct Client {
    intern: jmap_client::client::Client,
}

impl Client {
    pub async fn new() -> Self {
        let url = std::fs::read_to_string("/tmp/url.txt").unwrap();
        let password = std::fs::read_to_string("/tmp/password.txt").unwrap();
        let allow_redirects = std::fs::read_to_string("/tmp/redirects.txt").unwrap();

        let intern = jmap_client::client::Client::new()
            .credentials(("test", password.as_str()))
            .follow_redirects([allow_redirects])
            .connect(&url)
            .await
            .unwrap();

        Self { intern }
    }

    pub async fn get_mailboxes(&mut self) -> Vec<String> {
        self.intern
            .mailbox_query(None::<Filter>, None::<Vec<_>>)
            .await
            .unwrap();

        todo!()
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}
