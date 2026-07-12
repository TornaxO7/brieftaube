// mod mailbox;

use crate::{backend::Account, ui::MailboxId};
use jmap_client::{client::Client, mailbox::Mailbox};
use std::collections::HashMap;

pub struct Mailboxes {
    inner: HashMap<MailboxId, Mailbox>,
    state: String,
}

impl Mailboxes {
    pub async fn new(client: &Client) -> Self {
        let mut request = client.build();
        request.get_mailbox().ids::<[_; 1], String>(None::<[_; 1]>);
        let mut response = request.send_get_mailbox().await.unwrap();
        let state = response.take_state();

        let inner = response
            .take_list()
            .into_iter()
            .map(|mailbox| {
                let id = mailbox.id().unwrap().to_string();
                (id, mailbox)
            })
            .collect();

        Self { inner, state }
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
                self.inner.insert(id.to_string(), mailbox);
            }
        }

        // create
        {
            let mut request = client.build();
            request.get_mailbox().ids(Some(response.created()));
            let mut response = request.send_get_mailbox().await.unwrap();

            let mailboxes: Vec<(MailboxId, Mailbox)> = response
                .take_list()
                .into_iter()
                .map(|mailbox| {
                    let id = mailbox.id().unwrap().to_string();
                    (id, mailbox)
                })
                .collect();

            self.inner.extend(mailboxes);
        }

        // destroy
        {
            let to_destroy = response.take_destroyed();

            self.inner.retain(|id, _mailbox| {
                to_destroy
                    .iter()
                    .map(|id| id.as_str())
                    .find(|&id_to_destroy| id == id_to_destroy)
                    .is_some()
            });
        }

        self.state = response.take_new_state();
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
                        let boxes = mailboxes.inner.values().cloned().collect::<Vec<Mailbox>>();
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
