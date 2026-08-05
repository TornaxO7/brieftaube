mod error;
mod impls;
mod store;
pub mod types;

pub use store::Store;

// methods which also communicate with the server
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
