mod mailbox;

use crate::{
    backend::{Account, mailboxes::mailbox::MailboxCtx},
    utils::ui::MailboxId,
};
use jmap_client::{client::Client, mailbox::Mailbox};
use std::collections::HashMap;

pub struct Mailboxes {
    mailboxes: Vec<MailboxCtx>,
    state: String,

    // helper data structures
    mapping: HashMap<MailboxId, usize>,
}

impl Mailboxes {
    pub async fn new(client: &Client) -> Self {
        let mut request = client.build();
        request.get_mailbox().ids::<[_; 1], String>(None::<[_; 1]>);
        let mut response = request.send_get_mailbox().await.unwrap();

        let state = response.take_state();
        let mapping = response
            .list()
            .iter()
            .enumerate()
            .map(|(idx, mailbox)| {
                let id = mailbox.id().unwrap().to_string();
                (id, idx)
            })
            .collect();

        let mailboxes = response
            .take_list()
            .into_iter()
            .map(|mailbox| MailboxCtx::new(mailbox))
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
                    self.mailboxes[idx] = MailboxCtx::new(mailbox);
                }
            }
        }

        // create
        {
            let mut request = client.build();
            request.get_mailbox().ids(Some(response.created()));
            let mut response = request.send_get_mailbox().await.unwrap();

            let mailboxes: Vec<MailboxCtx> = response
                .take_list()
                .into_iter()
                .map(|mailbox| mailbox::MailboxCtx::new(mailbox))
                .collect();

            self.mailboxes.extend(mailboxes);
        }

        // destroy
        {
            let to_destroy = response.take_destroyed();

            self.mailboxes.retain(|mailbox| {
                to_destroy
                    .iter()
                    .map(|id| id.as_str())
                    .find(|&id| id == mailbox.id())
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
                let id = mailbox.id().to_string();
                (id, idx)
            })
            .collect();
    }
}

impl Account {
    pub fn get_mailboxes(&self, state: &str) -> Option<(Vec<Mailbox>, String)> {
        let data = self.data.lock().unwrap();

        match data.mailboxes.as_ref() {
            Some(mailboxes_data) => {
                let state_changed = mailboxes_data.state != state;

                if state_changed {
                    let mailboxes = mailboxes_data
                        .mailboxes
                        .iter()
                        .map(|ctx| ctx.mailbox().clone())
                        .collect::<Vec<Mailbox>>();

                    Some((mailboxes, mailboxes_data.state.to_owned()))
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
