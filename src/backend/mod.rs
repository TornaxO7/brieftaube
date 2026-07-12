pub mod mailboxes;
pub mod root_mails;
pub mod thread;

use crate::{
    config::Config,
    ui::{MailboxId, ThreadId},
};
use jmap_client::client::Client;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::task::{JoinError, JoinSet};

#[derive(Default)]
struct Data {
    mailboxes: Option<mailboxes::Mailboxes>,
    root_mails: HashMap<MailboxId, root_mails::RootMails>,
    threads: HashMap<ThreadId, thread::Thread>,
}

pub struct Account {
    client: Arc<jmap_client::client::Client>,
    config: Config,
    data: Arc<Mutex<Data>>,
    tasks: Arc<Mutex<JoinSet<()>>>,
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

        Self {
            config,
            client: Arc::new(client),
            data: Arc::new(Mutex::new(Data::default())),
            tasks: Arc::new(Mutex::new(JoinSet::new())),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<(), JoinError>> {
        self.tasks.lock().unwrap().join_next().await
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
