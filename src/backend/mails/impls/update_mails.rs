use crate::backend::{Backend, MailUpdate, task_manager::TaskId};
use tracing::{error, warn};

impl Backend {
    pub fn set_mails_updates(&self, updates: Vec<MailUpdate>) {
        let updates_do_nothing = updates.iter().all(|update| update.is_empty());

        if updates.is_empty() || updates_do_nothing {
            return;
        }

        let client = self.client.clone();
        let store = self.store.clone();
        self.task_manager.spawn(TaskId::UpdateMail, async move {
            let mut response = {
                let mut request = client.build();

                let set_mail = request.set_email();

                for update in updates.iter() {
                    if !update.is_empty() {
                        let mail_to_update = set_mail.update(&update.id.0);

                        if let Some(keywords) = update.patch_keywords.as_ref() {
                            for (keyword, set) in keywords {
                                mail_to_update.keyword(keyword.to_string().as_str(), *set);
                            }
                        }

                        if let Some(new_mailboxes) = update.mailbox_ids.as_ref() {
                            for (mailbox_id, set) in new_mailboxes {
                                mail_to_update.mailbox_id(&mailbox_id.0, *set);
                            }
                        }
                    }
                }

                match request.send_set_email().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!(
                            "Couldn't send `Email/set` request to server, to update mails:\n{err}"
                        );
                        return;
                    }
                }
            };

            let mut store = store.lock().unwrap();

            store.mails.set_state(response.take_new_state());

            for update in updates {
                match response.updated(&update.id.0) {
                    Ok(None) => {}
                    Ok(Some(huh)) => warn!(
                        "The server sent an unexpected response mail:\n{huh:?}\nCould you please create an issue? :>"
                    ),
                    Err(err) => {
                        error!("Couldn't update mail:\n{err}");
                        continue;
                    }
                }
                store.mails.update(update);
            }
        });
    }
}
