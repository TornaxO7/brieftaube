mod mailbox_data;

use crate::{backend::Account, ui::MailboxId};
use jmap_client::{client::Client, core::set::SetObject};
pub use mailbox_data::MailboxData;
use std::collections::HashMap;
use tracing::trace;

pub struct Mailboxes {
    inner: HashMap<MailboxId, MailboxData>,
    state: String,
}

impl Mailboxes {
    pub async fn new(client: &Client) -> color_eyre::Result<Self> {
        let mut request = client.build();
        request.get_mailbox().ids::<[_; 1], String>(None::<[_; 1]>);
        let mut response = request.send_get_mailbox().await?;
        let state = response.take_state();

        let inner = response
            .take_list()
            .into_iter()
            .map(|mailbox| {
                let id = mailbox.id().unwrap().to_string();
                (id, mailbox.into())
            })
            .collect();

        Ok(Self { inner, state })
    }
}

impl Account {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_mailboxes(&self, state: &str) -> Option<(Vec<MailboxData>, String)> {
        match self.data.try_lock() {
            Ok(data) => match data.mailboxes.as_ref() {
                Some(mailboxes) => {
                    let state_changed = mailboxes.state != state;

                    if state_changed {
                        trace!("State changed!");
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
            Err(_already_locked) => None,
        }
    }
}

impl Account {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn init_mailboxes(&self) {
        let data = self.data.clone();
        let client = self.client.clone();

        trace!("Init mailboxes");
        self.tasks.lock().unwrap().spawn(async move {
            let mut data = data.lock().await;
            let mailboxes = &mut data.mailboxes;

            if mailboxes.is_none() {
                let new_mailboxes = Mailboxes::new(&client).await?;

                *mailboxes = Some(new_mailboxes);
            }

            Ok(())
        });
    }

    pub fn update_mailbox_sort_order(&self, id: MailboxId, new_order: u32) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut data = data.lock().await;
            let mailboxes = data.mailboxes.as_mut().unwrap();

            let mut request = client.build();
            request.set_mailbox().update(&id).sort_order(new_order);
            let mut response = request.send_set_mailbox().await?;

            mailboxes.inner.get_mut(&id).unwrap().sort_order = new_order;
            mailboxes.state = response.take_new_state();

            Ok(())
        });
    }

    // pub fn fetch_changes(&self) {
    //     let mailboxes_are_not_set = { self.data.lock().unwrap().mailboxes.is_none() };
    //     if mailboxes_are_not_set {
    //         self.init_mailboxes();
    //         return;
    //     }

    //     let client = self.client.clone();
    //     let data = self.data.clone();

    //     self.tasks.lock().unwrap().spawn(async move {
    //         fetch_changes(client, data).await;
    //     });
    // }

    pub fn create_mailbox(&self, mut mailbox: MailboxData) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut data = data.lock().await;
            let mailboxes = data.mailboxes.as_mut().unwrap();

            let mut request = client.build();

            let id = request
                .set_mailbox()
                .create()
                .parent_id(mailbox.parent_id.clone())
                .name(mailbox.name.clone())
                .sort_order(mailbox.sort_order)
                .is_subscribed(true)
                .create_id()
                .unwrap();

            let mut response = request.send_set_mailbox().await?;
            mailbox.id = id.clone();

            mailboxes.inner.insert(id, mailbox);
            mailboxes.state = response.take_new_state();

            Ok(())
        });
    }

    pub fn destroy_mailbox(&self, id: MailboxId) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let mut data = data.lock().await;
            let mailboxes = data.mailboxes.as_mut().unwrap();

            let mut request = client.build();
            request
                .set_mailbox()
                .destroy([&id])
                .arguments()
                .on_destroy_remove_emails(false);
            let mut response = request.send_set_mailbox().await?;

            mailboxes.inner.remove(&id);
            mailboxes.state = response.take_new_state();

            Ok(())
        });
    }
}

// async fn fetch_changes(client: Arc<Client>, data: Arc<Mutex<Data>>) {
//     let current_state = {
//         let data = data.lock().unwrap();
//         let mailboxes = data.mailboxes.as_ref().unwrap();
//         mailboxes.state.clone()
//     };
//     let mut request = client.build();
//     request.changes_mailbox(current_state);

//     let mut response = request.send_changes_mailbox().await.unwrap();

//     // update
//     {
//         let mut request = client.build();
//         request.get_mailbox().ids(Some(response.updated()));
//         let mut response = request.send_get_mailbox().await.unwrap();

//         let mut data = data.lock().unwrap();
//         let mailboxes = data.mailboxes.as_mut().unwrap();
//         for mailbox in response.take_list() {
//             let id = mailbox.id().unwrap();
//             mailboxes.inner.insert(id.to_string(), mailbox.into());
//         }
//     }

//     // create
//     {
//         let mut request = client.build();
//         request.get_mailbox().ids(Some(response.created()));
//         let mut response = request.send_get_mailbox().await.unwrap();

//         let new_mailboxes: Vec<(MailboxId, MailboxData)> = response
//             .take_list()
//             .into_iter()
//             .map(|mailbox| {
//                 let id = mailbox.id().unwrap().to_string();
//                 (id, mailbox.into())
//             })
//             .collect();

//         let mut data = data.lock().unwrap();
//         let mailboxes = data.mailboxes.as_mut().unwrap();
//         mailboxes.inner.extend(new_mailboxes);
//     }

//     // destroy
//     {
//         let to_destroy = response.take_destroyed();

//         let mut data = data.lock().unwrap();
//         let mailboxes = data.mailboxes.as_mut().unwrap();
//         mailboxes.inner.retain(|id, _mailbox| {
//             to_destroy
//                 .iter()
//                 .map(|id| id.as_str())
//                 .find(|&id_to_destroy| id == id_to_destroy)
//                 .is_none()
//         });
//     }

//     {
//         let mut data = data.lock().unwrap();
//         let mailboxes = data.mailboxes.as_mut().unwrap();
//         mailboxes.state = response.take_new_state();
//     }
// }
