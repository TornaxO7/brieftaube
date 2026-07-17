mod layers;
mod mailbox_data;
mod task_error;

use crate::utils::MailboxId;
use jmap_client::{
    URI,
    client::Client,
    core::{session::Capabilities, set::SetObject},
    mailbox::Role,
};
pub use layers::{Layer, Layers};
pub use mailbox_data::MailboxData;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use task_error::*;
use tokio::task::{JoinError, JoinSet};
use tracing::{debug, warn};

const NEW_SORT_ORDER_SIZE: u32 = 32;

pub struct Data {
    pub layers: Layers,
    state: String,
}

pub struct Backend {
    client: Arc<Client>,
    pub data: Arc<Mutex<Option<Data>>>,
    tasks: Arc<Mutex<JoinSet<Result<(), TaskError>>>>,
}

impl Backend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            data: Arc::new(Mutex::new(None)),
            tasks: Arc::new(Mutex::new(JoinSet::new())),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<Result<(), TaskError>, JoinError>> {
        self.tasks.lock().unwrap().join_next().await
    }

    pub fn is_initialised(&self) -> bool {
        self.data.lock().unwrap().is_some()
    }
}

// methods which also communicate with the server
impl Backend {
    pub fn init(&self) {
        let client = self.client.clone();
        let data = self.data.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut request = client.build();
            request.get_mailbox().ids::<[_; 1], String>(None::<[_; 1]>);
            let mut response = request.send_get_mailbox().await?;

            let state = response.take_state();
            let layers = {
                let mailboxes: Vec<MailboxData> = response
                    .take_list()
                    .into_iter()
                    .map(|mailbox| MailboxData::from(mailbox))
                    .collect();

                Layers::new(mailboxes)
            };

            let mut data = data.lock().unwrap();
            let new_data = Data { layers, state };
            *data = Some(new_data);

            Ok(())
        });
    }

    pub fn destroy_mailboxes(&self, ids: Vec<MailboxId>) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut request = client.build();
            request
                .set_mailbox()
                .destroy(&ids)
                .arguments()
                .on_destroy_remove_emails(false);
            let mut response = request.send_set_mailbox().await?;
            let mut errors = Vec::new();

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect("Data is initialised");

            for id in ids.into_iter() {
                match response.destroyed(&id) {
                    Ok(()) => {
                        data.layers.remove_mailbox(id);
                    }
                    Err(err) => {
                        let mailbox = data.layers.get_mailbox(&id).unwrap();
                        let name = mailbox.name.clone();

                        errors.push((name, err));
                    }
                }
            }

            if !errors.is_empty() {
                Err(TaskError::DestroyMailboxes { errors })
            } else {
                Ok(())
            }
        });
    }

    pub fn set_new_order(&self, id: MailboxId, new_order: u32) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut request = client.build();
            request.set_mailbox().update(&id).sort_order(new_order);
            request.send_set_mailbox().await?;

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect("Is initialised");
            data.layers.set_sort_order(id, new_order);

            Ok(())
        });
    }

    pub fn create_mailbox(&self, parent: Option<MailboxId>, name: String) {
        debug!("Creating mailbox: '{}' with parent '{:?}'.", &name, &parent);
        if !self.is_initialised() {
            return;
        }

        let client = self.client.clone();
        let data = self.data.clone();
        let caps = self.mail_capability();

        self.tasks.lock().unwrap().spawn(async move {
            {
                let guard = data.lock().unwrap();
                let data = guard.as_ref().expect("Is initialised");

                if data.layers.depth() > caps.max_mailbox_depth() {
                    Err(ErrorCreateMailbox::ReachedMaxDepth(
                        caps.max_mailbox_depth(),
                    ))
                } else if name.len() > caps.max_size_mailbox_name() {
                    Err(ErrorCreateMailbox::NameTooLong(
                        caps.max_size_mailbox_name(),
                    ))
                } else if data
                    .layers
                    .get_current_layer()
                    .contains_mailbox_name(name.as_str())
                {
                    Err(ErrorCreateMailbox::NameAlreadyUsed(name.clone()))
                } else {
                    Ok(())
                }
            }?;

            let mut new_mailbox = {
                let sort_order = {
                    let guard = data.lock().unwrap();
                    let data = guard.as_ref().expect("Is initialised");
                    let layer = data.layers.get_layer(&parent);
                    layer
                        .mailboxes
                        .iter()
                        .map(|mailbox| mailbox.sort_order)
                        .max()
                        .map(|biggest_sort_order| {
                            (biggest_sort_order + 1).next_multiple_of(NEW_SORT_ORDER_SIZE)
                        })
                        .unwrap_or(NEW_SORT_ORDER_SIZE)
                };

                MailboxData {
                    name: name.clone(),
                    role: Role::None,
                    sort_order,
                    parent_id: parent.clone(),
                    ..Default::default()
                }
            };

            let mut request = client.build();
            let id = request
                .set_mailbox()
                .create()
                .name(&new_mailbox.name)
                .parent_id(new_mailbox.parent_id.as_ref())
                .role(new_mailbox.role.clone())
                .sort_order(new_mailbox.sort_order)
                .is_subscribed(true)
                .create_id()
                .unwrap();
            let mut response = request.send_set_mailbox().await?;
            let mut create_mailbox = response.created(&id)?;
            new_mailbox.id = create_mailbox.take_id();

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect("Data is initialised");
            data.layers.add_mailbox(new_mailbox);
            data.state = response.take_new_state();

            Ok(())
        });
    }

    pub fn normalize_sort_order(&self) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let ids: Vec<MailboxId> = {
                let guard = data.lock().unwrap();
                let data = guard.as_ref().expect("Is initialised");
                let layer = data.layers.get_current_layer();
                layer
                    .mailboxes
                    .iter()
                    .map(|mailbox| mailbox.id.clone())
                    .collect()
            };

            let new_sort_orders: HashMap<MailboxId, u32> = ids
                .into_iter()
                .enumerate()
                .map(|(idx, id)| (id, (idx + 1) as u32 * NEW_SORT_ORDER_SIZE))
                .collect();

            // send changes
            {
                let mut request = client.build();
                let set_mailbox = request.set_mailbox();
                for (id, new_sort_order) in new_sort_orders.iter() {
                    set_mailbox.update(id).sort_order(*new_sort_order);
                }
                let mut response = request.send_set_mailbox().await?;

                // check that everything worked fine
                for id in new_sort_orders.keys() {
                    response.updated(id)?;
                }
            }

            // apply changes to local
            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect("Is initialised");
            let layer = data.layers.get_current_layer_mut();
            for mailbox in layer.mailboxes.iter_mut() {
                match new_sort_orders.get(&mailbox.id).cloned() {
                    Some(new_order) => mailbox.sort_order = new_order,
                    None => warn!("The mailbox '{}' didn't exist, before fetching its sort order... skipping new sort order for it.", &mailbox.name),
                }
            }

            Ok(())
        });
    }

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

// ui functions
impl Backend {
    pub fn select_next_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let current_layer = data.layers.get_current_layer_mut();
            current_layer.state.select_next();
        }
    }

    pub fn select_previous_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let current_layer = data.layers.get_current_layer_mut();
            current_layer.state.select_previous();
        }
    }

    pub fn activate_selected_entry(&self) -> Option<MailboxId> {
        let mut guard = self.data.lock().unwrap();
        guard
            .as_mut()
            .and_then(|data| data.layers.open_selected_entry())
    }

    pub fn go_back(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.layers.go_up_one_level();
        }
    }

    pub fn can_set_sort_order(&self) -> Option<bool> {
        let guard = self.data.lock().unwrap();
        guard.as_ref().map(|data| {
            let layer = data.layers.get_current_layer();
            !layer.selected_parent()
        })
    }

    pub fn get_selected_mailbox(&self) -> Option<MailboxId> {
        let guard = self.data.lock().unwrap();
        guard
            .as_ref()
            .map(|data| data.layers.get_current_layer())
            .and_then(|layer| layer.get_selected_mailbox())
            .map(|mailbox| mailbox.id.clone())
    }

    pub fn get_parent_mailbox(&self) -> Option<MailboxId> {
        let guard = self.data.lock().unwrap();
        guard
            .as_ref()
            .map(|data| data.layers.get_current_layer())
            .and_then(|layer| layer.mailbox_owner.clone())
    }
}
