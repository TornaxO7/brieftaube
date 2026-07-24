pub mod mailbox;
pub mod mails;
pub mod threads;

use crate::config::Config;
use jmap_client::client::Client;
use std::{rc::Rc, sync::Arc};

type GetState = String;

pub struct Backend {
    pub client: Arc<jmap_client::client::Client>,

    config: Rc<Config>,

    mailboxes: Rc<mailbox::MailboxBackend>,
    mails: Rc<mails::MailsBackend>,
}

impl Backend {
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
            mailboxes: Rc::new(mailbox::MailboxBackend::new(client.clone())),
            mails: Rc::new(mails::MailsBackend::new(client.clone())),
            config,
        }
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }

    pub fn has_tasks_running(&self) -> bool {
        self.mailboxes.has_tasks_running() || self.mails.has_tasks_running()
    }

    pub async fn has_changed(&self) {
        tokio::select! {
            _ = self.mailboxes.has_changed(), if self.mailboxes.has_tasks_running() => {
                self.mailboxes.pop_task();
            }
            _ = self.mails.has_changed(), if self.mails.has_tasks_running() => {
                self.mails.pop_task();
            }
        }
    }
}
