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

// pub async fn request_update(
//     &self,
//     mailboxes: Vec<MailboxUpdate>,
// ) -> Result<(), MailboxUpdateError> {
//     if mailboxes.is_empty() {
//         return Ok(());
//     }

//     self.validate_mailbox_updates(&mailboxes)?;

//     let mut response = {
//         let mut request = self.client.build();
//         let set_mailbox = request.set_mailbox();

//         for mailbox in mailboxes.iter() {
//             let update = set_mailbox.update(&mailbox.id);
//             if let Some(name) = &mailbox.name {
//                 update.name(name);
//             }

//             if let Some(role) = mailbox.role.clone() {
//                 update.role(role);
//             }

//             if let Some(sort_order) = mailbox.sort_order.clone() {
//                 update.sort_order(sort_order);
//             }

//             if let Some(parent_id) = mailbox.parent_id.clone() {
//                 update.parent_id(parent_id);
//             }
//         }

//         request.send_set_mailbox().await?
//     };

//     let mut cache = self.cache.lock().unwrap();

//     for mailbox in mailboxes {
//         if response.updated(mailbox.id.as_str())?.is_some() {
//             warn!(
//                 "Server also wanted some updates... but it... shouldn't. Please restart the client, just to be sure."
//             );
//         }
//         cache.update(mailbox);
//     }

//     Ok(())
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

// helpers
// impl MailboxBackend {
//     fn mail_capability(&self) -> jmap_client::email::MailCapabilities {
//         let id = self.client.default_account_id();

//         match self
//             .client
//             .session()
//             .account(id)
//             .unwrap()
//             .capability(URI::Mail.as_ref())
//             .unwrap()
//             .clone()
//         {
//             Capabilities::Mail(cap) => cap,
//             _ => unreachable!(),
//         }
//     }

//     fn validate_mailbox_updates<'a, M>(
//         &self,
//         mailboxes: &'a [M],
//     ) -> Result<(), MailboxValidationError>
//     where
//         &'a M: Into<MailboxValidate>,
//     {
//         let cache = self.cache.lock().unwrap();
//         let caps = self.mail_capability();

//         for mailbox in mailboxes {
//             let MailboxValidate {
//                 name, parent_id, ..
//             } = mailbox.into();

//             if let Some(name) = name.as_ref() {
//                 let min = 1;
//                 let max = caps.max_size_mailbox_name();

//                 if !(min < name.len() && name.len() <= max) {
//                     return Err(MailboxValidationError::NameTooLong { max });
//                 }
//             }

//             if let Some(parent_id) = parent_id.as_ref() {
//                 let max = caps.max_mailbox_depth();
//                 if cache.depth_of(parent_id) + 1 > max {
//                     return Err(MailboxValidationError::MaxDepthExceeded { max });
//                 }
//             }

//             if let Some(parent_id) = parent_id.as_ref()
//                 && let Some(name) = name.as_ref()
//             {
//                 if cache.contains_mailbox_name(&parent_id, &name) {
//                     return Err(MailboxValidationError::DuplicateName { name: name.clone() });
//                 }
//             }
//         }

//         Ok(())
//     }
// }
