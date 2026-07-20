mod cache;
mod error;
pub mod types;

use crate::backend::mailbox::types::{Children, Entry};
use cache::Cache;
use error::MailboxValidationError;
use jmap_client::{
    URI,
    client::Client,
    core::{session::Capabilities, set::SetObject},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::task::{JoinError, JoinHandle};
use tracing::error;
use types::{MailboxData, MailboxId, MailboxNew, MailboxUpdate, MailboxValidate, SortOrder};

const DATA_INITIALISED_MSG: &str = "Is initialised";

pub struct MailboxBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Option<Cache>>>,
    tasks: Mutex<VecDeque<JoinHandle<()>>>,
}

impl MailboxBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(None)),
            tasks: Mutex::new(VecDeque::with_capacity(16)),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<(), JoinError>> {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.front_mut().unwrap();
        Some(task.await)
    }

    pub fn pop_task(&self) {
        self.tasks
            .lock()
            .unwrap()
            .pop_front()
            .expect("There are tasks.");
    }

    pub fn cache_is_initialised(&self) -> bool {
        self.cache.lock().unwrap().is_some()
    }
}

// methods which also communicate with the server
impl MailboxBackend {
    pub fn init(&self) {
        if self.cache_is_initialised() {
            // TODO: Request `mailbox/changes`
            return;
        }

        let client = self.client.clone();
        let cache = self.cache.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let response = {
                    let mut request = client.build();
                    request
                        .get_mailbox()
                        .ids::<[_; 1], String>(None::<[_; 1]>)
                        .properties(MailboxData::PROPERTIES);

                    match request.send_get_mailbox().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request mailboxes from server: {err}");
                            return;
                        }
                    }
                };

                *cache.lock().unwrap() = Some(Cache::new(response));
            }));
    }

    pub fn destroy_mailboxes(&self, ids: Vec<MailboxId>) {
        if !self.cache_is_initialised() || ids.is_empty() {
            return;
        }

        let cache = self.cache.clone();
        let client = self.client.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let current_state = {
                        let guard = cache.lock().unwrap();
                        let cache = guard.as_ref().expect(DATA_INITIALISED_MSG);
                        cache.get_current_state()
                    };

                    let mut request = client.build();
                    let set_mailbox = request.set_mailbox();
                    set_mailbox.destroy(&ids).if_in_state(current_state);
                    set_mailbox.arguments().on_destroy_remove_emails(false);

                    match request.send_set_mailbox().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request server to destroy mailboxes: {err}");
                            return;
                        }
                    }
                };

                let mut guard = cache.lock().unwrap();
                let cache = guard.as_mut().expect(DATA_INITIALISED_MSG);
                cache.set_state(response.take_new_state());

                for id in ids.into_iter() {
                    match response.destroyed(&id) {
                        Ok(()) => {
                            cache.remove_mailbox(&id);
                        }
                        Err(err) => match cache.get_mailbox(&id) {
                            Some(mailbox) => {
                                let name = mailbox.name.clone();
                                error!("Couldn't destroy the mailbox '{name}': {err}");
                            }
                            None => {
                                error!("Couldn't destroy mailbox:\n{err}");
                            }
                        },
                    }
                }
            }));
    }

    pub fn update_mailboxes(&self, mailboxes: Vec<MailboxUpdate>) {
        if !self.cache_is_initialised() || mailboxes.is_empty() {
            return;
        }

        if let Err(errors) = self.validate_mailboxes(&mailboxes) {
            for error in errors {
                error!("Can't update mailbox: {}", error);
            }
            return;
        }

        let cache = self.cache.clone();
        let client = self.client.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let mut response = {
                    let current_state = {
                        let guard = cache.lock().unwrap();
                        guard
                            .as_ref()
                            .expect(DATA_INITIALISED_MSG)
                            .get_current_state()
                    };

                    let mut request = client.build();
                    let set_mailbox = request.set_mailbox().if_in_state(current_state);

                    for mailbox in mailboxes.iter() {
                        let u = set_mailbox.update(&mailbox.id);
                        if let Some(name) = &mailbox.name {
                            u.name(name);
                        }

                        if let Some(role) = mailbox.role.clone() {
                            u.role(role);
                        }

                        if let Some(sort_order) = mailbox.sort_order.clone() {
                            u.sort_order(sort_order);
                        }

                        if let Some(parent_id) = mailbox.parent_id.clone() {
                            u.parent_id(parent_id);
                        }
                    }

                    match request.send_set_mailbox().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request server to update the mailboxes:\n{err}");
                            return;
                        }
                    }
                };

                let mut guard = cache.lock().unwrap();
                let cache = guard.as_mut().expect(DATA_INITIALISED_MSG);
                cache.set_state(response.take_new_state());

                for mailbox in mailboxes {
                    match response.updated(&mailbox.id) {
                        Ok(_) => {
                            cache.update_mailbox(mailbox);
                        }
                        Err(err) => match cache.get_mailbox(&mailbox.id) {
                            Some(mailbox) => {
                                let name = mailbox.name.clone();
                                error!("Couldn't update the mailbox of '{name}':\n{err}");
                            }
                            None => {
                                error!("Couldn't update a mailbox:\n{err}");
                            }
                        },
                    };
                }
            }));
    }

    pub fn create_mailboxes(&self, mailboxes: Vec<MailboxNew>) {
        if !self.cache_is_initialised() || mailboxes.is_empty() {
            return;
        }

        {
            if let Err(errors) = self.validate_mailboxes(&mailboxes) {
                for error in errors {
                    error!("Can't create new mailbox: {}", error);
                }
                return;
            }
        }

        let client = self.client.clone();
        let cache = self.cache.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(async move {
                let (mut response, tmp_ids) = {
                    let current_state = {
                        let guard = cache.lock().unwrap();
                        guard
                            .as_ref()
                            .expect(DATA_INITIALISED_MSG)
                            .get_current_state()
                    };

                    let mut tmp_ids = Vec::with_capacity(mailboxes.len());
                    let mut request = client.build();
                    let set_mailbox = request.set_mailbox().if_in_state(current_state);

                    for mailbox in mailboxes.iter() {
                        let c = set_mailbox.create();
                        c.name(&mailbox.name);
                        c.parent_id(mailbox.parent_id.clone());

                        if let Some(role) = mailbox.role.clone() {
                            c.role(role);
                        }

                        if let Some(sort_order) = mailbox.sort_order {
                            c.sort_order(sort_order);
                        }

                        tmp_ids.push(c.create_id().unwrap());
                    }

                    match request.send_set_mailbox().await {
                        Ok(r) => (r, tmp_ids),
                        Err(err) => {
                            error!("Couldn't send request to server to create mailbox: {err}");
                            return;
                        }
                    }
                };

                let mut guard = cache.lock().unwrap();
                let cache = guard.as_mut().expect(DATA_INITIALISED_MSG);
                cache.set_state(response.take_new_state());

                for (mailbox, tmp_id) in mailboxes.into_iter().zip(tmp_ids.into_iter()) {
                    match response.created(&tmp_id) {
                        Ok(mut server) => {
                            let id = server.take_id();
                            let name = server
                                .name()
                                .map(ToString::to_string)
                                .unwrap_or(mailbox.name);
                            let role = server.role();
                            let sort_order = server.sort_order();
                            let parent_id = mailbox.parent_id;
                            let unread_mails = server.unread_emails();

                            let mailbox = MailboxData {
                                id,
                                name,
                                role,
                                sort_order,
                                parent_id,
                                unread_mails,
                            };

                            cache.add_new_mailbox(mailbox);
                        }
                        Err(err) => {
                            error!("Couldn't create mailbox '{}': {err}", mailbox.name);
                            return;
                        }
                    };
                }
            }));
    }

    // pub fn rename_mailboxes(&self, mapping: Vec<(MailboxId, String)>) {
    //     if !self.is_cache_initialised() {
    //         return;
    //     }

    //     // TODO: add check if that's even possible

    //     let cache = self.cache.clone();
    //     let client = self.client.clone();

    //     self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
    //         let mut response = {
    //             let current_state = {
    //                 let guard = cache.lock().unwrap();
    //                 guard.as_ref().expect(DATA_INITIALISED_MSG).get_current_state()
    //             };

    //             let mut request = client.build();
    //             {
    //                 let set_mailbox = request.set_mailbox().if_in_state(current_state);
    //                 for map in mapping.iter() {
    //                     set_mailbox.update(&map.0).name(&map.1);
    //                 }
    //             }

    //             match request.send_set_mailbox().await {
    //                 Ok(r) => r,
    //                 Err(err) => {
    //                     error!("Couldn't request server to rename mailboxes: {err}");
    //                     return;
    //                 }
    //             }
    //         };

    //         let mut guard = cache.lock().unwrap();
    //         let cache = guard.as_mut().expect(DATA_INITIALISED_MSG);
    //         for (id, new_name) in mapping.into_iter() {
    //             let mailbox = cache.get_mailbox_mut(&id).expect("Mailbox exists");

    //             match response.updated(&id) {
    //                 Ok(server) => {
    //                     if let Some(server) = server {
    //                         warn!("Server requested other changes: {:#?}", server);
    //                     }
    //                     mailbox.name = new_name;
    //                 }
    //                 Err(err) => {
    //                     let old_name = &mailbox.name;
    //                     let new_name = &new_name;
    //                     warn!("Couldn't rename mailbox '{old_name}' to '{new_name}': {err}\nSkipping this mailbox.");
    //                     continue;
    //                 }
    //             }

    //         }

    //     }));
    // }

    // pub fn move_mailbox_down(&self, id: MailboxId) {
    //     if !self.is_cache_initialised() {
    //         return;
    //     }

    //     let new_order = {
    //         let guard = self.cache.lock().unwrap();
    //         let data = guard.as_ref().expect(DATA_INITIALISED_MSG);

    //         // validate
    //         let layer = data.tree.get_layer_containing_mailbox(&id);
    //         let idx = layer
    //             .mailboxes
    //             .iter()
    //             .enumerate()
    //             .find_map(|(idx, mailbox)| (mailbox.id == id).then_some(idx))
    //             .unwrap();
    //         let is_at_bottom = idx == layer.mailboxes.len() - 1;
    //         if is_at_bottom {
    //             return;
    //         }

    //         let below1 = layer.mailboxes[idx + 1].sort_order;
    //         match layer.mailboxes.get(idx + 2) {
    //             Some(below2_mailbox) => {
    //                 let below2 = below2_mailbox.sort_order;
    //                 below1 + (below2 - below1) / 2
    //             }
    //             None => (below1 + 1).next_multiple_of(NEW_SORT_ORDER_SIZE),
    //         }
    //     };

    //     self.set_new_order(id, new_order);
    // }

    // pub fn move_mailboxes_to(&self, ids: Vec<MailboxId>, new_parent: Option<MailboxId>) {
    //     if !self.is_cache_initialised() {
    //         return;
    //     }

    //     struct MailboxToMove {
    //         id: MailboxId,
    //         old_name: String,
    //         new_name: Option<String>,
    //     }

    //     let filtered_ids: Vec<MailboxToMove> = {
    //         let guard = self.cache.lock().unwrap();
    //         let data = guard.as_ref().expect(DATA_INITIALISED_MSG);

    //         let mut filtered_ids = Vec::with_capacity(ids.len());
    //         let new_parent_layer = data.tree.get_layer(&new_parent);
    //         for id in ids.into_iter() {
    //             let mailbox = data.tree.get_mailbox(&id).unwrap();
    //             let old_name = mailbox.name.clone();

    //             if new_parent_layer.contains_mailbox(&id) {
    //                 continue;
    //             }

    //             let new_name = if new_parent_layer.contains_mailbox_name(&mailbox.name) {
    //                 Some(format!("{}-1", &mailbox.name))
    //             } else {
    //                 None
    //             };

    //             filtered_ids.push(MailboxToMove {
    //                 id,
    //                 old_name,
    //                 new_name,
    //             });
    //         }

    //         filtered_ids
    //     };

    //     let cache = self.cache.clone();
    //     let client = self.client.clone();

    //     self.tasks
    //         .lock()
    //         .unwrap()
    //         .push_back(tokio::spawn(async move {
    //             let mut response = {
    //                 let mut request = client.build();
    //                 let set_mailbox = request.set_mailbox();
    //                 for MailboxToMove { id, new_name, .. } in filtered_ids.iter() {
    //                     match new_name {
    //                         Some(name) => set_mailbox
    //                             .update(id)
    //                             .parent_id(new_parent.clone())
    //                             .name(name),
    //                         None => set_mailbox.update(id).parent_id(new_parent.clone()),
    //                     };
    //                 }
    //                 match request.send_set_mailbox().await {
    //                     Ok(r) => r,
    //                     Err(err) => {
    //                         error!("Couldn't send request to server for moving mailboxes: {err}");
    //                         return;
    //                     }
    //                 }
    //             };

    //             let mut guard = cache.lock().unwrap();
    //             let cache = guard.as_mut().expect(DATA_INITIALISED_MSG);
    //             cache.set_state(response.take_new_state());
    //             for MailboxToMove {
    //                 id,
    //                 old_name,
    //                 new_name,
    //             } in filtered_ids.into_iter()
    //             {
    //                 match response.updated(&id) {
    //                     Ok(server_changes) => {
    //                         let old = cache.tree.get_mailbox(&id).unwrap().clone();
    //                         let mut new = old.clone();
    //                         new.parent_id = new_parent.clone();
    //                         if let Some(name) = new_name {
    //                             new.name = name;
    //                         }

    //                         if let Some(server) = server_changes {
    //                             if let Some(name) = server.name() {
    //                                 new.name = name.to_string()
    //                             }

    //                             if let Some(parent_id) = server.parent_id() {
    //                                 new.parent_id = Some(parent_id.to_string());
    //                             }
    //                         }

    //                         // update its layer
    //                         {
    //                             let mut layer = cache.tree.remove_layer(&Some(old.id.clone()));
    //                             layer.mailbox_owner = Some(new.id.clone());
    //                             cache.tree.insert_layer(Some(new.id.clone()), layer);
    //                         }

    //                         // update its old and new parent layer
    //                         if old.parent_id != new.parent_id {
    //                             let prev_layer = cache.tree.get_layer_mut(&old.parent_id);
    //                             prev_layer
    //                                 .mailboxes
    //                                 .extract_if(.., |mailbox| mailbox.id == old.id)
    //                                 .for_each(drop);

    //                             let new_layer = cache.tree.get_layer_mut(&new.parent_id);
    //                             new_layer.mailboxes.push(new);
    //                             new_layer.sort_mailboxes();
    //                         }
    //                     }
    //                     Err(err) => {
    //                         error!("Couldn't update '{old_name}': {err}");
    //                         continue;
    //                     }
    //                 }
    //             }
    //         }));
    // }

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

// helpers
impl MailboxBackend {
    fn validate_mailboxes<'a, M>(
        &self,
        mailboxes: &'a [M],
    ) -> Result<(), Vec<MailboxValidationError>>
    where
        &'a M: Into<MailboxValidate>,
    {
        let guard = self.cache.lock().unwrap();
        let cache = guard.as_ref().expect(DATA_INITIALISED_MSG);
        let caps = self.mail_capability();
        let mut errors = Vec::with_capacity(mailboxes.len());

        for mailbox in mailboxes {
            let MailboxValidate {
                name,
                role: _,
                sort_order: _,
                parent_id,
            } = mailbox.into();

            if let Some(name) = name.as_ref() {
                let min = 1;
                let max = caps.max_size_mailbox_name();

                if !(min < name.len() && name.len() < max) {
                    errors.push(MailboxValidationError::NameTooLong { max });
                }
            }

            if let Some(parent_id) = parent_id.as_ref() {
                let max = caps.max_mailbox_depth();
                if cache.depth_of(parent_id) + 1 > max {
                    errors.push(MailboxValidationError::MaxDepthExceeded { max });
                }
            }

            if let Some(parent_id) = parent_id.as_ref()
                && let Some(name) = name.as_ref()
            {
                if cache.contains_mailbox_name(&parent_id, &name) {
                    errors.push(MailboxValidationError::DuplicateName {
                        name: name.to_string(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// methods for `state.rs`
impl MailboxBackend {
    pub fn get_children_ids(&self, parent_id: &Option<MailboxId>) -> Option<Vec<Entry>> {
        let guard = self.cache.lock().unwrap();
        guard
            .as_ref()
            .map(|cache| cache.get_children(parent_id).to_vec())
    }

    pub fn get_children_sort_order(
        &self,
        parent_id: &Option<MailboxId>,
    ) -> Option<Vec<Children<SortOrder>>> {
        let guard = self.cache.lock().unwrap();
        guard.as_ref().map(|cache| {
            cache
                .get_children(parent_id)
                .iter()
                .map(|entry| match entry {
                    Entry::This => Children::This,
                    Entry::Child(id) => {
                        let mailbox = cache.get_mailbox(id).unwrap();
                        Children::Child(id.clone(), mailbox.sort_order)
                    }
                })
                .collect()
        })
    }

    pub fn get_children_names(
        &self,
        parent_id: &Option<MailboxId>,
    ) -> Option<Vec<Children<String>>> {
        let guard = self.cache.lock().unwrap();
        guard.as_ref().map(|cache| {
            cache
                .get_children(parent_id)
                .iter()
                .map(|entry| match entry {
                    Entry::This => Children::This,
                    Entry::Child(id) => {
                        let mailbox = cache.get_mailbox(id).unwrap();

                        Children::Child(id.clone(), mailbox.name.clone())
                    }
                })
                .collect()
        })
    }

    pub fn get_mailbox_name(&self, id: &MailboxId) -> Option<String> {
        let guard = self.cache.lock().unwrap();
        guard
            .as_ref()
            .and_then(|cache| cache.get_mailbox(id).map(|mailbox| mailbox.name.clone()))
    }

    pub fn get_children(
        &self,
        parent_id: &Option<MailboxId>,
    ) -> Option<Vec<Children<MailboxData>>> {
        let guard = self.cache.lock().unwrap();
        guard.as_ref().map(|cache| {
            cache
                .get_children(parent_id)
                .iter()
                .map(|entry| match entry {
                    Entry::This => Children::This,
                    Entry::Child(id) => {
                        let mailbox = cache.get_mailbox(id).unwrap();

                        Children::Child(id.clone(), mailbox.clone())
                    }
                })
                .collect()
        })
    }
}
