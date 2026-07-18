mod backend;
mod mail_data;

use crate::utils::MailboxId;
pub use backend::{Data, RootMailsBackend};
use jmap_client::client::Client;
pub use mail_data::RootMailData;
use std::{collections::HashMap, rc::Rc, sync::Arc};

pub struct RootMailsManager {
    backends: HashMap<MailboxId, Rc<RootMailsBackend>>,
    selected: Option<MailboxId>,
}

impl RootMailsManager {
    pub fn new() -> Self {
        Self {
            backends: HashMap::with_capacity(16),
            selected: None,
        }
    }

    pub fn get_backend(&mut self, id: MailboxId, client: Arc<Client>) -> Rc<RootMailsBackend> {
        self.selected = Some(id.clone());

        self.backends
            .entry(id.clone())
            .or_insert(Rc::new(RootMailsBackend::new(client, id.clone())))
            .clone()
    }

    pub fn has_tasks_running(&self) -> bool {
        let Some(selected) = self.selected.as_ref() else {
            return false;
        };

        let backend = self.backends.get(selected).unwrap();
        backend.has_tasks_running()
    }

    pub async fn has_changed(&self) {
        if let Some(selected) = self.selected.as_ref() {
            let backend = self.backends.get(selected).unwrap();
            backend.has_changed().await;
        }
    }

    pub fn pop_task(&mut self) {
        if let Some(selected) = self.selected.as_ref() {
            let backend = self.backends.get_mut(selected).unwrap();
            backend.pop_task();
        }
    }
}
