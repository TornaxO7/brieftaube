mod mailbox_data;

use crate::{backend::Account, ui::MailboxId};
use jmap_client::client::Client;
pub use mailbox_data::MailboxData;
use std::collections::HashMap;

pub struct Mailboxes {
    inner: HashMap<MailboxId, MailboxData>,
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
                (id, mailbox.into())
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
                self.inner.insert(id.to_string(), mailbox.into());
            }
        }

        // create
        {
            let mut request = client.build();
            request.get_mailbox().ids(Some(response.created()));
            let mut response = request.send_get_mailbox().await.unwrap();

            let mailboxes: Vec<(MailboxId, MailboxData)> = response
                .take_list()
                .into_iter()
                .map(|mailbox| {
                    let id = mailbox.id().unwrap().to_string();
                    (id, mailbox.into())
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

    pub fn get_mailboxes(&self, state: &str) -> Option<(Vec<MailboxData>, String)> {
        match self.data.try_lock() {
            Ok(data) => match data.mailboxes.as_ref() {
                Some(mailboxes) => {
                    let state_changed = mailboxes.state != state;

                    if state_changed {
                        let boxes = mailboxes
                            .inner
                            .values()
                            .cloned()
                            .collect::<Vec<MailboxData>>();
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

    pub fn update_mailbox_sort_order(&self, mailboxes: Vec<(MailboxId, u32)>) {
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut request = client.build();
            let set_mailbox = request.set_mailbox();

            for (id, new_sort_order) in mailboxes {
                set_mailbox.update(id).sort_order(new_sort_order);
            }

            request.send_set_mailbox().await.unwrap();
        });
    }
}
