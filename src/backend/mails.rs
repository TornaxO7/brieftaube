use jmap_client::{client::Client, email::Email};

pub struct Mails {
    mails: Vec<Email>,
    state: String,
}

impl Mails {
    pub async fn new(client: &Client) -> Self {
        todo!()
    }
}
