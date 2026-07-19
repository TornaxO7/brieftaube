use crate::config::Config;
use jmap_client::client::Client;
use std::{rc::Rc, sync::Arc};

// TODO: Rename to `Backends`
pub struct Account {
    pub client: Arc<jmap_client::client::Client>,

    pub mailboxes: Rc<crate::mailboxes::Backend>,
    pub mail_lists: crate::mail_list::MailListManager,
}

impl Account {
    pub async fn new() -> Self {
        let config = Rc::new(Config::load().unwrap());

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
            mailboxes: Rc::new(crate::mailboxes::Backend::new(
                client.clone(),
                config.clone(),
            )),
            mail_lists: crate::mail_list::MailListManager::new(),
        }
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }
}
