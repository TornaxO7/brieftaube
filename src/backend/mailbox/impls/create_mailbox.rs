use crate::backend::{
    Backend, MailboxData, MailboxId, MailboxNew, mailbox::error::MailboxValidationError,
};
use jmap_client::core::set::SetObject;

#[derive(thiserror::Error, Debug)]
pub enum CreateMailboxError {
    #[error("Validation error: {0}")]
    Validation(#[from] MailboxValidationError),

    #[error("From server: {0}")]
    Jmap(#[from] jmap_client::Error),
}

impl Backend {
    pub async fn create_mailbox(&self, new: MailboxNew) -> Result<MailboxId, CreateMailboxError> {
        self.validate_mailbox(&new)?;

        let (mut response, tmp_id) = {
            let new = new.clone();
            let mut request = self.client.build();

            let create = request.set_mailbox().create().name(new.name);

            if let Some(role) = new.role {
                create.role(role);
            }

            if let Some(sort_order) = new.sort_order {
                create.sort_order(sort_order);
            }

            if let Some(parent_id) = new.parent_id {
                create.parent_id(Some(parent_id));
            }

            let tmp_id = create.create_id().unwrap();
            let response = request.send_set_mailbox().await?;

            (response, tmp_id)
        };

        let new_mailbox = {
            let mut server_mailbox = response.created(&tmp_id)?;

            MailboxData {
                id: MailboxId(server_mailbox.take_id()),
                name: new.name,
                role: new.role.unwrap_or(server_mailbox.role()),
                sort_order: new.sort_order.unwrap_or(server_mailbox.sort_order()),
                unread_mails: 0,
                parent_id: new.parent_id,
                total_threads: 0,
            }
        };

        let new_id = new_mailbox.id.clone();
        let mut store = self.store.lock().unwrap();
        store.mailbox.add(new_mailbox);
        Ok(new_id)
    }
}
