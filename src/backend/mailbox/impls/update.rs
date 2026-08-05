use jmap_client::{URI, core::session::Capabilities};
use tracing::{error, warn};

use crate::backend::{
    Backend, MailboxUpdate, MailboxValidate,
    mailbox::error::{MailboxValidationError}, task_manager::TaskId,
};

impl Backend {
    pub fn update_mailboxes(
        &self,
        mailboxes: Vec<MailboxUpdate>,
    ) {
        if mailboxes.is_empty() {
            return ;
        }

        if let Err(err) = self.validate_mailbox_updates(&mailboxes) {
            error!("Can't update mailbox:\n{err}");
            return;
        }

        let client = self.client.clone();
        let store = self.store.clone();

        self.task_manager.spawn(TaskId::UpdateMailbox, async move {
         let mut response = {
            let mut request = client.build();
            let set_mailbox = request.set_mailbox();

            for mailbox in mailboxes.iter() {
                let update = set_mailbox.update(&mailbox.id);
                if let Some(name) = &mailbox.name {
                    update.name(name);
                }

                if let Some(role) = mailbox.role.clone() {
                    update.role(role);
                }

                if let Some(sort_order) = mailbox.sort_order.clone() {
                    update.sort_order(sort_order);
                }

                if let Some(parent_id) = mailbox.parent_id.clone() {
                    update.parent_id(parent_id);
                }
            }

            match request.send_set_mailbox().await {
                Ok(r) => r,
                Err(err) => {
                    error!("Couldn't send request to update mailbox:\n{err}");
                    return;
                }
            }
        };

        let mut store = store.lock().unwrap();

        for mailbox in mailboxes {
            match response.updated(mailbox.id.as_str()) {
                Ok(None) => {},
                Ok(Some(_)) =>{
                    warn!(
                        "Server also wanted some updates... but it... shouldn't. Please restart the client, just to be sure."
                    );
                }
                Err(err) => {
                    error!("Couldn't update mailbox:\n{err}");
                    continue;
                }
            }
            store.mailbox.update(mailbox);
        }   
        });

        

    }

    fn mail_capability(&self) -> jmap_client::email::MailCapabilities {
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

    fn validate_mailbox_updates<'a, M>(
        &self,
        mailboxes: &'a [M],
    ) -> Result<(), MailboxValidationError>
    where
        &'a M: Into<MailboxValidate>,
    {
        let store = self.store.lock().unwrap();
        let mailbox_store = &store.mailbox;
        let caps = self.mail_capability();

        for mailbox in mailboxes {
            let MailboxValidate {
                name, parent_id, ..
            } = mailbox.into();

            if let Some(name) = name.as_ref() {
                let min = 1;
                let max = caps.max_size_mailbox_name();

                if !(min < name.len() && name.len() <= max) {
                    return Err(MailboxValidationError::NameTooLong { max });
                }
            }

            if let Some(parent_id) = parent_id.as_ref() {
                let max = caps.max_mailbox_depth();
                if mailbox_store.depth_of(parent_id) + 1 > max {
                    return Err(MailboxValidationError::MaxDepthExceeded { max });
                }
            }

            if let Some(parent_id) = parent_id.as_ref()
                && let Some(name) = name.as_ref()
            {
                if mailbox_store.contains_mailbox_name(&parent_id, &name) {
                    return Err(MailboxValidationError::DuplicateName { name: name.clone() });
                }
            }
        }

        Ok(())
    }
}
