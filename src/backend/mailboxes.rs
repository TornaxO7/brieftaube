// mod mailbox;

use crate::{backend::Account, ui::MailboxId};
use jmap_client::{client::Client, mailbox::Mailbox};
use std::collections::HashMap;

pub struct Mailboxes {
    inner: Vec<Mailbox>,
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

        let mailboxes = response.take_list();

        Self {
            inner: mailboxes,
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
                    self.inner[idx] = mailbox;
                }
            }
        }

        // create
        {
            let mut request = client.build();
            request.get_mailbox().ids(Some(response.created()));
            let mut response = request.send_get_mailbox().await.unwrap();

            let mailboxes = response.take_list();
            self.inner.extend(mailboxes);
        }

        // destroy
        {
            let to_destroy = response.take_destroyed();

            self.inner.retain(|mailbox| {
                to_destroy
                    .iter()
                    .map(|id| id.as_str())
                    .find(|&id| id == mailbox.id().unwrap())
                    .is_some()
            });
        }

        self.refresh_mapping();

        self.state = response.take_new_state();
    }

    fn refresh_mapping(&mut self) {
        self.mapping = self
            .inner
            .iter()
            .enumerate()
            .map(|(idx, mailbox)| {
                let id = mailbox.id().unwrap().to_string();
                (id, idx)
            })
            .collect();
    }

    fn get_mut_mailbox(&mut self, id: &MailboxId) -> &mut Mailbox {
        let idx = self.mapping.get(id).unwrap();
        self.inner.get_mut(*idx).unwrap()
    }

    fn get_mailbox(&self, id: &MailboxId) -> &Mailbox {
        let idx = self.mapping.get(id).unwrap();
        self.inner.get(*idx).unwrap()
    }
}

impl Account {
    pub fn init_mailboxes(&self) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mailboxes_arent_initalised = { data.lock().unwrap().mailboxes.is_none() };

            if mailboxes_arent_initalised {
                let mailboxes = Mailboxes::new(&client).await;

                data.lock().unwrap().mailboxes = Some(mailboxes);
            }
        });
    }

    pub fn get_mailboxes(&self, state: &str) -> Option<(Vec<Mailbox>, String)> {
        match self.data.try_lock() {
            Ok(data) => match data.mailboxes.as_ref() {
                Some(mailboxes) => {
                    let state_changed = mailboxes.state != state;

                    if state_changed {
                        let boxes = mailboxes.inner.clone();

                        Some((boxes, mailboxes.state.to_owned()))
                    } else {
                        None
                    }
                }
                None => None,
            },
            Err(std::sync::TryLockError::WouldBlock) => None,
            Err(std::sync::TryLockError::Poisoned(err)) => unreachable!("{:?}", err),
        }
    }
}
