pub mod mailbox;
pub mod mails;
pub mod threads;

use crate::config::Config;
use jmap_client::client::Client;
use std::{rc::Rc, sync::Arc};

type GetState = String;

pub struct Backend {
    config: Rc<Config>,

    mailboxes: mailbox::MailboxBackend,
    mails: mails::MailsBackend,
    threads: threads::ThreadsBackend,
}

/// Methods needed for `main.rs`
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
            mailboxes: mailbox::MailboxBackend::new(client.clone()),
            mails: mails::MailsBackend::new(client.clone()),
            threads: threads::ThreadsBackend::new(client.clone()),
            config,
        }
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

/// Methods for states.
impl Backend {}
