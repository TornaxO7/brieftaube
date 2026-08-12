use crate::backend::{Backend, MailboxUpdate, mailbox::error::MailboxValidationError};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum MailboxUpdateError {
    #[error("Can't update mailbox:\n{0}")]
    Validation(#[from] MailboxValidationError),

    #[error(transparent)]
    Jmap(#[from] jmap_client::Error),
}

impl Backend {
    pub async fn update_mailboxes(
        &self,
        mailboxes: Vec<MailboxUpdate>,
    ) -> Result<(), MailboxUpdateError> {
        if mailboxes.is_empty() {
            return Ok(());
        }

        for update in mailboxes.iter() {
            self.validate_mailbox(update)?;
        }

        let mut response = {
            let mut request = self.client.build();
            let set_mailbox = request.set_mailbox();
            for mailbox in mailboxes.iter() {
                mailbox.set_request(set_mailbox);
            }

            request.send_set_mailbox().await?
        };

        let mut store = self.store.lock().unwrap();

        for mailbox in mailboxes {
            match response.updated(mailbox.id.as_str()) {
                Ok(None) => {}
                Ok(Some(_)) => {
                    warn!(concat![
                        "Server also wanted some updates... but it... shouldn't.\n",
                        "Please restart the client, just to be sure."
                    ]);
                }
                Err(err) => {
                    error!("Couldn't update mailbox:\n{err}");
                    continue;
                }
            }
            store.mailbox.update(mailbox);
        }

        Ok(())
    }
}
