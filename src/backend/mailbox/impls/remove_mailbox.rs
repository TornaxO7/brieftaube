use crate::backend::{Backend, MailboxId};

#[derive(thiserror::Error, Debug)]
pub enum RemoveMailboxError {
    #[error("Can't remove mailbox: It's not empty.")]
    NotEmpty,

    #[error("Can't remove mailbox. It has child mailboxes.")]
    HasChildMailboxes,

    #[error(transparent)]
    Jmap(#[from] jmap_client::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum RemoveMailboxOption {
    /// Only remove mailbox if it's empty
    Empty,
    /// Also remove mails from this mailbox
    Mails,
    // /// Remove all mails and mailboxes in this mailbox.
    // Recursive,
}

impl Backend {
    pub async fn remove_mailbox(
        &self,
        id: &MailboxId,
        option: RemoveMailboxOption,
    ) -> Result<(), RemoveMailboxError> {
        self.validate_remove_mailbox(id, option)?;

        let mut response = {
            let mut request = self.client.build();

            let remove_mails = match option {
                RemoveMailboxOption::Empty => false,
                RemoveMailboxOption::Mails => true,
            };

            request
                .set_mailbox()
                .destroy([&id.0])
                .arguments()
                .on_destroy_remove_emails(remove_mails);
            request.send_set_mailbox().await?
        };

        response.destroyed(&id.0)?;

        match option {
            RemoveMailboxOption::Empty => {}
            RemoveMailboxOption::Mails => {
                let mut store = self.store.lock().unwrap();

                let root_mails = store.mailbox.get_root_mails(&id).unwrap().clone();

                for root_mail_id in root_mails.ids.iter() {
                    let root_mail = store.mails.remove(root_mail_id).unwrap();
                    let thread_mails = store.threads.remove(&root_mail.thread_id).unwrap();

                    for thread_mail in thread_mails.iter() {
                        store.mails.remove(thread_mail);
                    }
                }

                store.mailbox.remove(&id);
            }
        };

        Ok(())
    }

    fn validate_remove_mailbox(
        &self,
        id: &MailboxId,
        option: RemoveMailboxOption,
    ) -> Result<(), RemoveMailboxError> {
        let store = self.store.lock().unwrap();

        let has_child_mailboxes = !store
            .mailbox
            .get_children(&Some(id.clone()))
            .unwrap()
            .is_empty();
        let has_mails = !store.mailbox.get_root_mails(id).unwrap().ids.is_empty();

        match option {
            RemoveMailboxOption::Empty => {
                if has_child_mailboxes || has_mails {
                    return Err(RemoveMailboxError::NotEmpty);
                }
            }
            RemoveMailboxOption::Mails => {
                if has_child_mailboxes {
                    return Err(RemoveMailboxError::HasChildMailboxes);
                }
            }
        };

        Ok(())
    }
}
