mod layer;
mod mailbox_data;

use crate::utils::MailboxId;
use jmap_client::client::Client;
pub use layer::{Layer, Layers};
pub use mailbox_data::MailboxData;
use std::sync::{Arc, Mutex};
use tokio::task::{JoinError, JoinSet};
use tracing::error;

pub struct Data {
    pub layers: Layers,
    state: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error(transparent)]
    Client(#[from] jmap_client::Error),
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
}

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

    // pub fn can_set_sort_order(&self) -> bool {
    //     let guard = self.data.lock().unwrap();
    //     let Some(data) = guard.as_ref() else {
    //         return false;
    //     };
    //     let layer = data.layers.get_current_layer();
    //     !layer.selected_parent()
    // }

    pub fn destroy_selected_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let layer = data.layers.get_current_layer();

            match layer.get_selected_mailbox() {
                Some(_mailbox) => {
                    todo!()
                }
                None => {
                    error!(
                        "Can't destroy mailbox: You can't select the parent mailbox to destroy it."
                    );
                }
            }
        }
    }

    // pub fn set_new_order(&self, new_order: u32) {
    //     let mut guard = self.data.lock().unwrap();
    //     if let Some(data) = guard.as_mut() {
    //         match data.layers.set_sort_order(new_order) {
    //             Some(_id) => {
    //                 todo!()
    //                 // pub fn update_mailbox_sort_order(&self, id: MailboxId, new_order: u32) {
    //                 //         let data = self.data.clone();
    //                 //         let client = self.client.clone();

    //                 //         self.tasks.lock().unwrap().spawn(async move {
    //                 //             let mut data = data.lock().await;
    //                 //             let mailboxes = data.mailboxes.as_mut().unwrap();

    //                 //             let mut request = client.build();
    //                 //             request.set_mailbox().update(&id).sort_order(new_order);
    //                 //             let mut response = request.send_set_mailbox().await?;

    //                 //             mailboxes.inner.get_mut(&id).unwrap().sort_order = new_order;
    //                 //             mailboxes.state = response.take_new_state();

    //                 //             Ok(())
    //                 //         });
    //                 //     }
    //             }
    //             None => unreachable!(
    //                 "A check should have happened before that the current selected mailbox isn't a parent directory..."
    //             ),
    //         }
    //     }
    // }

    pub fn create_mailbox(&self, _name: String) {
        let mut guard = self.data.lock().unwrap();
        if let Some(_data) = guard.as_mut() {
            // validate
            // {
            //     let caps = self.account.mail_capability();

            //     let msg = {
            //         if layers.depth() > caps.max_mailbox_depth() {
            //             Some(format!(
            //                 "Max mailbox depth reached for the mail server :( You can't create another sub-mailbox in the current mailbox. The maximum depth is {} for the server.",
            //                 caps.max_mailbox_depth()
            //             ))
            //         } else if value.len() > caps.max_size_mailbox_name() {
            //             Some(format!(
            //                 "The mailbox name is too long. It can be at most {} characters long.",
            //                 caps.max_size_mailbox_name()
            //             ))
            //         } else if layers
            //             .get_current_layer()
            //             .contains_mailbox_name(value.as_str())
            //         {
            //             Some(format!(
            //                 "There's already a mailbox in the current mailbox with the name '{}'.",
            //                 &value
            //             ))
            //         } else {
            //             None
            //         }
            //     };

            //     if let Some(msg) = msg {
            //         error!("Can't create mailbox: {}", msg);
            //         return;
            //     }
            // }

            // let layer = layers.get_current_layer();
            // let new_mailbox_data = MailboxData {
            //     name: value.clone(),
            //     parent_id: layer.mailbox_owner.clone(),
            //     sort_order: layer
            //         .mailboxes
            //         .iter()
            //         .map(|mailbox| mailbox.sort_order)
            //         .max()
            //         .map(|biggest_sort_order| biggest_sort_order + 1)
            //         .unwrap_or(0),
            //     ..Default::default()
            // };

            // self.tasks.lock().unwrap().spawn(async move {
            //             let mut data = data.lock().await;
            //             let mailboxes = data.mailboxes.as_mut().unwrap();

            //             let mut request = client.build();

            //             let id = request
            //                 .set_mailbox()
            //                 .create()
            //                 .parent_id(mailbox.parent_id.clone())
            //                 .name(mailbox.name.clone())
            //                 .sort_order(mailbox.sort_order)
            //                 .role(Role::None)
            //                 .is_subscribed(true)
            //                 .create_id()
            //                 .unwrap();

            //             let mut response = request.send_set_mailbox().await?;
            //             let mut created_mailbox = response.created(&id)?;
            //             mailbox.id = created_mailbox.take_id();

            //             mailboxes.inner.insert(mailbox.id.clone(), mailbox.clone());
            //             mailboxes.state = response.take_new_state();

            //             info!("Successfully created mailbox '{}'.", mailbox.name.clone());

            //             Ok(())
            //         });        }
        }
        todo!()
    }

    pub fn destroy_mailbox(&self, _mailbox: MailboxData) {
        todo!()
    }
}
