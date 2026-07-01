use crate::{backend, ui::MailboxId};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::debug;

pub struct State {
    account: Arc<backend::Account>,

    rx_mails: mpsc::Receiver<Vec<Email>>,
    tx_mails: Arc<mpsc::Sender<Vec<Email>>>,

    mails: HashMap<MailboxId, Option<Vec<Email>>>,
    selected_mailbox_id: Option<MailboxId>,
    list_state: tui_widget_list::ListState,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        let (tx_mails, rx_mails) = mpsc::channel(1);

        Self {
            account,

            rx_mails,
            tx_mails: Arc::new(tx_mails),

            mails: HashMap::new(),

            selected_mailbox_id: None,
            list_state: tui_widget_list::ListState::default(),
        }
    }

    pub fn open_mailbox(&mut self, mailbox_id: MailboxId) {
        self.selected_mailbox_id = Some(mailbox_id);
    }

    // pub fn get_mailbox_names(&mut self) -> Option<Vec<String>> {
    //     if let Some(mailboxes) = &self.mailboxes {
    //         Some(
    //             mailboxes
    //                 .iter()
    //                 .map(|mailbox| mailbox.name().unwrap().to_string())
    //                 .collect(),
    //         )
    //     } else {
    //         match self.rx_mailboxes.try_recv() {
    //             Ok(mailboxes) => self.mailboxes = Some(mailboxes),
    //             Err(mpsc::error::TryRecvError::Empty) => {}
    //             Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
    //         }
    //         None
    //     }
    // }

    pub fn get_mails<'a>(&mut self) -> Option<Vec<Email>> {
        if let Some(selected_mailbox_id) = self.selected_mailbox_id.clone() {
            if let Some(possible_mails) = self.mails.get(&selected_mailbox_id) {
                if let Some(mails) = possible_mails {
                    return Some(mails.clone());
                } else {
                    // We are still waiting for te mails to come in
                    match self.rx_mails.try_recv() {
                        Ok(mails) => {
                            self.mails
                                .insert(selected_mailbox_id.to_string(), Some(mails.clone()));
                            return Some(mails);
                        }
                        Err(mpsc::error::TryRecvError::Empty) => todo!(),
                        Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
                    }
                }
            } else {
                // indicate that fetching the mail started
                self.mails.insert(selected_mailbox_id.to_string(), None);

                let account = self.account.clone();

                let tx = self.tx_mails.clone();
                tokio::spawn(async move {
                    // TODO: Error handling
                    let mut query_response = {
                        let mut request = account.client.build();

                        let query = request.query_email();
                        query.arguments().collapse_threads(true);
                        query.filter(jmap_client::email::query::Filter::in_mailbox(
                            selected_mailbox_id,
                        ));
                        // query.filter(jmap_client::core::query::Filter::and([
                        //     jmap_client::email::query::Filter::in_mailbox(mailbox_id).into(),
                        //     jmap_client::core::query::Filter::not([
                        //         jmap_client::email::query::Filter::From {
                        //             value: account.address.clone(),
                        //         },
                        //     ]),
                        // ]));

                        request.send_query_email().await.unwrap()
                    };

                    // TODO: Listen to changes
                    let ids = query_response.take_ids();

                    let mut mails = {
                        let mut request = account.client.build();
                        request.get_email().ids(Some(ids));
                        // TODO: Error handling
                        request.send_get_email().await.unwrap()
                    };

                    tx.send(mails.take_list()).await.unwrap();
                });
            }
        }

        None
    }

    pub fn get_mail(&self, selected_mail_idx: usize) -> Option<Email> {
        if let Some(selected_mailbox_id) = &self.selected_mailbox_id {
            if let Some(mails) = self.mails.get(selected_mailbox_id) {
                if let Some(mails) = mails {
                    return Some(mails[selected_mail_idx].clone());
                }
            }
            debug!(
                "Didn't fetch mails from mailbox with id '{}'",
                selected_mailbox_id
            );
        } else {
            debug!("No mailboxes available yet");
        }

        None
    }

    pub fn select_next_mail(&mut self) {
        self.list_state.next();
    }

    pub fn select_previous_mail(&mut self) {
        self.list_state.previous();
    }

    pub fn selected_mail_list_idx(&self) -> Option<usize> {
        self.list_state.selected
    }

    pub fn get_mail_list_state_mut(&mut self) -> &mut tui_widget_list::ListState {
        &mut self.list_state
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
