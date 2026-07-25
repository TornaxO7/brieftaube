mod cache;
mod error;
pub mod types;

use crate::backend::{Backend, mailbox::types::ParentMailboxId, task_manager::TaskId};
use cache::Cache;
use jmap_client::client::Client;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, instrument};
use types::{MailboxData, MailboxId};

pub struct MailboxBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
}

impl MailboxBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }
}

/// Helper functions
impl MailboxBackend {
    fn cache_is_initialised(&self, parent: &ParentMailboxId) -> bool {
        self.cache.lock().unwrap().is_initialised(parent)
    }
}

// methods which also communicate with the server
impl MailboxBackend {
    #[instrument(skip(self))]
    async fn request_mailboxes_query(
        &self,
        parent: ParentMailboxId,
    ) -> Result<(), jmap_client::Error> {
        // debug!("Requesting child mailboxes of '{:?}'", &parent);
        if self.cache_is_initialised(&parent) {
            // debug!("Cache already contains child mailboxes of '{:?}'", &parent);
            // TODO: Request `mailbox/changes`
            return Ok(());
        }

        let mut response = {
            let mut request = self.client.build();

            let query_result = request
                .query_mailbox()
                .filter(jmap_client::mailbox::query::Filter::ParentId {
                    value: parent.clone().map(|id| id.0),
                })
                .result_reference();

            request
                .get_mailbox()
                .ids_ref(query_result)
                .properties(MailboxData::PROPERTIES);

            debug!("Send request.");
            request.send().await?
        };

        let mut mailbox_get_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_mailbox()?;

        let mut cache = self.cache.lock().unwrap();
        let child_mailboxes = mailbox_get_response.take_list();
        debug!(
            "Child mailboxes of '{:?}':\n{:#?}",
            &parent, &child_mailboxes
        );
        for mailbox in child_mailboxes {
            cache.add(MailboxData::from(mailbox));
        }

        cache.set_state(parent.clone(), mailbox_get_response.take_state());
        Ok(())
    }

    // pub async fn request_remove_mailbox(&self, ids: Vec<MailboxId>) {
    //     if !self.cache_is_initialised() || ids.is_empty() {
    //         return;
    //     }

    //     let mut response = {
    //         let current_state = {
    //             let cache = self.cache.lock().unwrap();
    //             cache.get_state()
    //         };

    //         let mut request = self.client.build();
    //         let set_mailbox = request.set_mailbox();
    //         set_mailbox.destroy(&ids).if_in_state(current_state);
    //         set_mailbox.arguments().on_destroy_remove_emails(false);

    //         match request.send_set_mailbox().await {
    //             Ok(r) => r,
    //             Err(err) => {
    //                 error!("Couldn't request server to destroy mailboxes: {err}");

    //                 match err {
    //                     jmap_client::Error::Method(method) => match method.p_type {
    //                         MethodErrorType::StateMismatch => {
    //                             self.request_mailboxes_get().await;
    //                         }
    //                         _ => {}
    //                     },
    //                     _ => {}
    //                 }
    //                 return;
    //             }
    //         }
    //     };

    //     let mut cache = self.cache.lock().unwrap();
    //     cache.set_state(response.take_new_state());
    //     for id in ids.into_iter() {
    //         match response.destroyed(&id) {
    //             Ok(()) => {
    //                 cache.remove(id);
    //             }
    //             Err(err) => match cache.get_data(&id) {
    //                 Some(mailbox) => {
    //                     let name = mailbox.name.clone();
    //                     error!("Couldn't destroy the mailbox '{name}': {err}");
    //                 }
    //                 None => {
    //                     error!("Couldn't destroy mailbox:\n{err}");
    //                 }
    //             },
    //         }
    //     }
    // }

    // pub async fn update_mailboxes(&self, mailboxes: Vec<MailboxUpdate>) {
    //     if !self.cache_is_initialised() || mailboxes.is_empty() {
    //         return;
    //     }

    //     if let Err(errors) = self.validate_mailboxes(&mailboxes) {
    //         for error in errors {
    //             error!("Can't update mailbox: {}", error);
    //         }
    //         return;
    //     }

    //     let mut response = {
    //         let current_state = {
    //             let cache = self.cache.lock().unwrap();
    //             cache.get_state()
    //         };

    //         let mut request = self.client.build();
    //         let set_mailbox = request.set_mailbox().if_in_state(current_state);

    //         for mailbox in mailboxes.iter() {
    //             let u = set_mailbox.update(&mailbox.id);
    //             if let Some(name) = &mailbox.name {
    //                 u.name(name);
    //             }

    //             if let Some(role) = mailbox.role.clone() {
    //                 u.role(role);
    //             }

    //             if let Some(sort_order) = mailbox.sort_order.clone() {
    //                 u.sort_order(sort_order);
    //             }

    //             if let Some(parent_id) = mailbox.parent_id.clone() {
    //                 u.parent_id(parent_id);
    //             }
    //         }

    //         match request.send_set_mailbox().await {
    //             Ok(r) => r,
    //             Err(err) => {
    //                 error!("Couldn't request server to update the mailboxes:\n{err}");

    //                 match err {
    //                     jmap_client::Error::Method(method) => match method.p_type {
    //                         MethodErrorType::StateMismatch => {
    //                             self.request_mailboxes_get().await;
    //                         }
    //                         _ => {}
    //                     },
    //                     _ => {}
    //                 }

    //                 return;
    //             }
    //         }
    //     };

    //     let mut cache = self.cache.lock().unwrap();
    //     cache.set_state(response.take_new_state());

    //     for mailbox in mailboxes {
    //         match response.updated(&mailbox.id) {
    //             Ok(_) => {
    //                 cache.update(mailbox);
    //             }
    //             Err(err) => match cache.get_data(&mailbox.id) {
    //                 Some(mailbox) => {
    //                     let name = mailbox.name.clone();
    //                     error!("Couldn't update the mailbox of '{name}':\n{err}");
    //                 }
    //                 None => {
    //                     error!("Couldn't update a mailbox:\n{err}");
    //                 }
    //             },
    //         };
    //     }
    // }

    // pub async fn create_mailboxes(&self, mailboxes: Vec<MailboxNew>) {
    //     if !self.cache_is_initialised() || mailboxes.is_empty() {
    //         return;
    //     }

    //     {
    //         if let Err(errors) = self.validate_mailboxes(&mailboxes) {
    //             for error in errors {
    //                 error!("Can't create new mailbox: {}", error);
    //             }
    //             return;
    //         }
    //     }

    //     let (mut response, tmp_ids) = {
    //         let current_state = {
    //             let cache = self.cache.lock().unwrap();
    //             cache.get_state()
    //         };

    //         let mut tmp_ids = Vec::with_capacity(mailboxes.len());
    //         let mut request = self.client.build();
    //         let set_mailbox = request.set_mailbox().if_in_state(current_state);

    //         for mailbox in mailboxes.iter() {
    //             let c = set_mailbox.create();
    //             c.name(&mailbox.name);
    //             c.parent_id(mailbox.parent_id.clone());

    //             if let Some(role) = mailbox.role.clone() {
    //                 c.role(role);
    //             }

    //             if let Some(sort_order) = mailbox.sort_order {
    //                 c.sort_order(sort_order);
    //             }

    //             tmp_ids.push(c.create_id().unwrap());
    //         }

    //         match request.send_set_mailbox().await {
    //             Ok(r) => (r, tmp_ids),
    //             Err(err) => {
    //                 error!("Couldn't request server to update the mailboxes:\n{err}");

    //                 match err {
    //                     jmap_client::Error::Method(method) => match method.p_type {
    //                         MethodErrorType::StateMismatch => {
    //                             self.request_mailboxes_get().await;
    //                         }
    //                         _ => {}
    //                     },
    //                     _ => {}
    //                 }

    //                 return;
    //             }
    //         }
    //     };

    //     let mut cache = self.cache.lock().unwrap();
    //     cache.set_state(response.take_new_state());

    //     for (mailbox, tmp_id) in mailboxes.into_iter().zip(tmp_ids.into_iter()) {
    //         match response.created(&tmp_id) {
    //             Ok(mut server) => {
    //                 let id = server.take_id();
    //                 let name = server
    //                     .name()
    //                     .map(ToString::to_string)
    //                     .unwrap_or(mailbox.name);
    //                 let role = server.role();
    //                 let sort_order = server.sort_order();
    //                 let parent_id = mailbox.parent_id;
    //                 let unread_mails = server.unread_emails();

    //                 let mailbox = MailboxData {
    //                     id,
    //                     name,
    //                     role,
    //                     sort_order,
    //                     parent_id,
    //                     unread_mails,
    //                 };

    //                 cache.add(mailbox);
    //             }
    //             Err(err) => {
    //                 error!("Couldn't create mailbox '{}': {err}", mailbox.name);
    //                 return;
    //             }
    //         };
    //     }
    // }

    // pub fn mail_capability(&self) -> jmap_client::email::MailCapabilities {
    //     let id = self.client.default_account_id();

    //     match self
    //         .client
    //         .session()
    //         .account(id)
    //         .unwrap()
    //         .capability(URI::Mail.as_ref())
    //         .unwrap()
    //         .clone()
    //     {
    //         Capabilities::Mail(cap) => cap,
    //         _ => unreachable!(),
    //     }
    // }
}

// helpers
// impl MailboxBackend {
//     fn validate_mailboxes<'a, M>(
//         &self,
//         mailboxes: &'a [M],
//     ) -> Result<(), Vec<MailboxValidationError>>
//     where
//         &'a M: Into<MailboxValidate>,
//     {
//         let cache = self.cache.lock().unwrap();
//         let caps = self.mail_capability();
//         let mut errors = Vec::with_capacity(mailboxes.len());

//         for mailbox in mailboxes {
//             let MailboxValidate {
//                 name,
//                 role: _,
//                 sort_order: _,
//                 parent_id,
//             } = mailbox.into();

//             if let Some(name) = name.as_ref() {
//                 let min = 1;
//                 let max = caps.max_size_mailbox_name();

//                 if !(min < name.len() && name.len() <= max) {
//                     errors.push(MailboxValidationError::NameTooLong { max });
//                 }
//             }

//             if let Some(parent_id) = parent_id.as_ref() {
//                 let max = caps.max_mailbox_depth();
//                 if cache.depth_of(parent_id) + 1 > max {
//                     errors.push(MailboxValidationError::MaxDepthExceeded { max });
//                 }
//             }

//             if let Some(parent_id) = parent_id.as_ref()
//                 && let Some(name) = name.as_ref()
//             {
//                 if cache.contains_mailbox_name(&parent_id, &name) {
//                     errors.push(MailboxValidationError::DuplicateName {
//                         name: name.to_string(),
//                     });
//                 }
//             }
//         }

//         if errors.is_empty() {
//             Ok(())
//         } else {
//             Err(errors)
//         }
//     }
// }

impl MailboxBackend {
    fn get_child_mailboxes(&self, parent: &ParentMailboxId) -> Option<Vec<MailboxId>> {
        let cache = self.cache.lock().unwrap();
        cache.get_children(parent).map(|children| children.to_vec())
    }

    fn get_mailbox_data(&self, id: &MailboxId) -> Option<Arc<MailboxData>> {
        let cache = self.cache.lock().unwrap();
        cache.get_data(id)
    }
}

impl Backend {
    pub fn mailbox_get_data(&self, id: &MailboxId) -> Option<Arc<MailboxData>> {
        self.mailboxes.get_mailbox_data(id)
    }

    #[instrument(skip(self))]
    pub fn mailboxes_get_children(&self, parent: ParentMailboxId) -> Option<Vec<MailboxId>> {
        let mailbox_backend = self.mailboxes.clone();

        self.mailboxes.get_child_mailboxes(&parent).or_else(|| {
            debug!("No child mailboxes available of '{:?}'", parent.clone());
            self.task_manager
                .spawn(TaskId::QueryChildMailboxes(parent.clone()), async move {
                    if let Err(err) = mailbox_backend
                        .request_mailboxes_query(parent.clone())
                        .await
                    {
                        error!("Couldn't query mailboxes:\n{err}");
                    }
                    debug!("Received child mailboxes of '{:?}'", parent.clone());
                });
            None
        })
    }
}
