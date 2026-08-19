use crate::backend::{Backend, MailboxId};

#[derive(thiserror::Error, Debug)]
pub enum RemoveMailboxError {
    #[error("Mailbox needs to be empty.")]
    NotEmpty,

    // #[error("Can't remove mailbox. It has child mailboxes.")]
    // HasChildMailboxes,
    #[error(transparent)]
    Jmap(#[from] jmap_client::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum RemoveMailboxOption {
    /// Only remove mailbox if it's empty
    Empty,
    // /// Also remove mails from this mailbox
    // Mails,
    // /// Remove all mails and mailboxes in this mailbox.
    // Recursive,
}

impl Backend {
    pub async fn remove_mailboxes(
        &self,
        ids: &[MailboxId],
        option: RemoveMailboxOption,
    ) -> Result<(), RemoveMailboxError> {
        let mut response = {
            let mut request = self.client.build();

            let remove_mails = match option {
                RemoveMailboxOption::Empty => false,
                // RemoveMailboxOption::Mails => true,
            };

            request
                .set_mailbox()
                .destroy(ids.iter().map(|id| &id.0))
                .arguments()
                .on_destroy_remove_emails(remove_mails);
            request.send_set_mailbox().await?
        };

        let mut store = self.store.lock().unwrap();
        for id in ids.iter() {
            response.destroyed(&id.0)?;
            store.mailbox.remove(id);
        }

        Ok(())
    }
}
