use crate::utils::ui::MailboxId;
use jmap_client::{client::Client, mailbox::Mailbox};
use std::collections::HashMap;

#[derive(Default)]
pub struct Mailboxes {
    mailboxes: Vec<Mailbox>,
    mapping: HashMap<MailboxId, usize>,
    state: String,
}

impl Mailboxes {
    pub async fn new(client: &Client) -> Self {
        let mut request = client.build();
        request.get_mailbox().ids::<[_; 1], String>(None::<[_; 1]>);
        let mut response = request.send_get_mailbox().await.unwrap();

        let state = response.take_state();
        let mailboxes = response.take_list();
        let mapping = mailboxes
            .iter()
            .enumerate()
            .map(|(idx, mailbox)| {
                let id = mailbox.id().unwrap().to_string();
                (id, idx)
            })
            .collect();

        Self {
            mailboxes,
            state,
            mapping,
        }
    }

    pub async fn fetch_changes(&mut self, client: &Client) {
        let mut request = client.build();
        request.changes_mailbox(&self.state);

        let mut response = request.send_changes_mailbox().await.unwrap();

        // update
        {
            let mut request = client.build();
            request.get_mailbox().ids(Some(response.updated()));
            let mut response = request.send_get_mailbox().await.unwrap();
            for mailbox in response.take_list() {
                let id = mailbox.id().unwrap();
                if let Some(idx) = self.mapping.get(id).cloned() {
                    self.mailboxes[idx] = mailbox;
                }
            }
        }

        // create
        {
            let mut request = client.build();
            request.get_mailbox().ids(Some(response.created()));
            let mut response = request.send_get_mailbox().await.unwrap();

            self.mailboxes.extend(response.take_list());
        }

        // destroy
        {
            let to_destroy = response.take_destroyed();

            self.mailboxes.retain(|mailbox| {
                let id = mailbox.id().unwrap();

                to_destroy
                    .iter()
                    .map(|id| id.as_str())
                    .find(|&destroy_id| destroy_id == id)
                    .is_some()
            });
        }

        self.refresh_mapping();

        self.state = response.take_new_state();
    }

    fn refresh_mapping(&mut self) {
        self.mapping = self
            .mailboxes
            .iter()
            .enumerate()
            .map(|(idx, mailbox)| {
                let id = mailbox.id().unwrap().to_string();
                (id, idx)
            })
            .collect();
    }
}
