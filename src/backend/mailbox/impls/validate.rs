use crate::backend::{Backend, MailboxValidate, mailbox::error::MailboxValidationError};
use jmap_client::{URI, core::session::Capabilities};

impl Backend {
    pub fn validate_mailbox<V>(&self, validate: V) -> Result<(), MailboxValidationError>
    where
        V: Into<MailboxValidate>,
    {
        let caps = self.mail_capability();
        let store = self.store.lock().unwrap();

        let MailboxValidate {
            name, parent_id, ..
        } = validate.into();

        if let Some(name) = name.as_ref() {
            let min = 1;
            let max = caps.max_size_mailbox_name();

            if !(min < name.len() && name.len() <= max) {
                return Err(MailboxValidationError::NameTooLong { max });
            }
        }

        if let Some(parent_id) = parent_id.as_ref() {
            let max = caps.max_mailbox_depth();
            if store.mailbox.depth_of(parent_id) + 1 > max {
                return Err(MailboxValidationError::MaxDepthExceeded { max });
            }
        }

        if let Some(parent_id) = parent_id.as_ref()
            && let Some(name) = name.as_ref()
        {
            if store.mailbox.contains_mailbox_name(&parent_id, &name) {
                return Err(MailboxValidationError::DuplicateName { name: name.clone() });
            }
        }

        Ok(())
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
}
