pub mod thread;

use crate::{config::Config, utils::ThreadId};
use jmap_client::client::Client;
use std::{collections::HashMap, rc::Rc, sync::Arc};
use tokio::task::{JoinError, JoinSet};

#[derive(Default)]
struct Data {
    pub threads: HashMap<ThreadId, thread::Thread>,
}

// TODO: Rename to `Backends`
pub struct Account {
    pub client: Arc<jmap_client::client::Client>,

    pub mailboxes: Rc<crate::mailboxes::Backend>,
    pub root_mails: crate::root_mails::RootMailsManager,
    data: Arc<tokio::sync::Mutex<Data>>,
    tasks: Arc<std::sync::Mutex<JoinSet<color_eyre::Result<()>>>>,
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
            root_mails: crate::root_mails::RootMailsManager::new(),
            data: Arc::new(tokio::sync::Mutex::new(Data::default())),
            tasks: Arc::new(std::sync::Mutex::new(JoinSet::new())),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty() && self.mailboxes.has_tasks_running()
    }

    pub async fn has_changed(&self) -> Option<Result<color_eyre::Result<()>, JoinError>> {
        self.tasks.lock().unwrap().join_next().await
    }

    pub fn address(&self) -> String {
        self.client.session().username().to_string()
    }
}
