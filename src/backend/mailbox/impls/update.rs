use crate::backend::{Backend, MailboxUpdate, task_manager::TaskId};
use tracing::{error, warn};

impl Backend {
    pub fn update_mailboxes(&self, mailboxes: Vec<MailboxUpdate>) {
        if mailboxes.is_empty() {
            return;
        }

        for update in mailboxes.iter() {
            if let Err(err) = self.validate_mailbox(update) {
                error!("Can't update mailbox:\n{err}");
                return;
            }
        }

        let client = self.client.clone();
        let store = self.store.clone();

        self.task_manager.spawn(TaskId::UpdateMailbox, async move {
            let mut response = {
                let mut request = client.build();
                let set_mailbox = request.set_mailbox();
                for mailbox in mailboxes.iter() {
                    mailbox.set_request(set_mailbox);
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
        });
    }
}
