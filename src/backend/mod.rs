pub mod mailboxes;
pub mod root_mails;
pub mod thread;

use crate::{
    config::Config,
    utils::{MailboxId, ThreadId},
};
use jmap_client::{URI, client::Client, core::session::Capabilities};
use std::{collections::HashMap, sync::Arc};
use tokio::task::{JoinError, JoinSet};

#[derive(Default)]
struct Data {
    mailboxes: Option<mailboxes::Mailboxes>,
    root_mails: HashMap<MailboxId, root_mails::RootMails>,
    threads: HashMap<ThreadId, thread::Thread>,
}

pub struct Account {
    client: Arc<jmap_client::client::Client>,
    _config: Config,
    data: Arc<tokio::sync::Mutex<Data>>,
    tasks: Arc<std::sync::Mutex<JoinSet<color_eyre::Result<()>>>>,
}

impl Account {
    pub async fn new() -> Self {
        let config = Config::load().unwrap();

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
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
            _config: config,
            client: Arc::new(client),
            data: Arc::new(tokio::sync::Mutex::new(Data::default())),
            tasks: Arc::new(std::sync::Mutex::new(JoinSet::new())),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<color_eyre::Result<()>, JoinError>> {
        self.tasks.lock().unwrap().join_next().await
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }

    // pub fn config(&self) -> &Config {
    //     &self.config
    // }
}

impl Account {
    pub fn mail_capability(&self) -> jmap_client::email::MailCapabilities {
        let id = self.client.default_account_id();

        match self
            .client
            .session()
            .account(id)
            .unwrap()
            .capability(URI::Mail.as_ref())
            .unwrap()
            .clone()
        {
            Capabilities::Mail(cap) => cap,
            _ => unreachable!(),
        }
    }
}
