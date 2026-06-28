use crate::backend;
use jmap_client::{email::Email, mailbox::Mailbox};
use ratatui::widgets;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::debug;

type MailboxId = String;

pub struct State {
    account: Arc<backend::Account>,

    rx_mailboxes: mpsc::Receiver<Vec<Mailbox>>,

    rx_mails: mpsc::Receiver<Vec<Email>>,
    tx_mails: Arc<mpsc::Sender<Vec<Email>>>,

    mailboxes: Option<Vec<Mailbox>>,
    /// Contains all mails for each mailbox.
    mails: HashMap<MailboxId, Option<Vec<Email>>>,

    mailbox_list_state: widgets::ListState,
    mail_list_state: tui_widget_list::ListState,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        let (tx_mailboxes, rx_mailboxes) = mpsc::channel(1);
        let (tx_mails, rx_mails) = mpsc::channel(1);

        let account2 = account.clone();
        tokio::spawn(async move {
            let mut mailbox_response = {
                let mut request = account2.client.build();
                request.get_mailbox().ids::<[_; 0], String>(None);
                // TODO: Error handling
                request.send_get_mailbox().await.unwrap()
            };

            tx_mailboxes
                .send(mailbox_response.take_list())
                .await
                .unwrap();

            // TODO: Listen for changes
            // let (mailbox_state, mails_state) = (mailboxes.state(), mails.state());
        });

        Self {
            account,

            rx_mailboxes,

            rx_mails,
            tx_mails: Arc::new(tx_mails),

            mailboxes: None,
            mails: HashMap::new(),

            mailbox_list_state: widgets::ListState::default(),
            mail_list_state: tui_widget_list::ListState::default(),
        }
    }

    pub fn get_mailbox_names(&mut self) -> Option<Vec<String>> {
        if let Some(mailboxes) = &self.mailboxes {
            Some(
                mailboxes
                    .iter()
                    .map(|mailbox| mailbox.name().unwrap().to_string())
                    .collect(),
            )
        } else {
            match self.rx_mailboxes.try_recv() {
                Ok(mailboxes) => self.mailboxes = Some(mailboxes),
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
            }
            None
        }
    }

    pub fn get_mails<'a>(&mut self, selected_mailbox_idx: usize) -> Option<Vec<Email>> {
        if let Some(mailboxes) = &self.mailboxes {
            let selected_mailbox = mailboxes.get(selected_mailbox_idx).expect("Mailbox exists");
            let selected_mailbox_id = selected_mailbox.id().unwrap();

            if let Some(possible_mails) = self.mails.get(selected_mailbox_id) {
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
                        Err(mpsc::error::TryRecvError::Empty) => {}
                        Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
                    }
                }
            } else {
                // indicate that fetching the mail started
                self.mails.insert(selected_mailbox_id.to_string(), None);

                let account = self.account.clone();
                let mailbox_id = selected_mailbox.id().unwrap().to_string();

                let tx = self.tx_mails.clone();
                tokio::spawn(async move {
                    // TODO: Error handling
                    let mut query_response = {
                        let mut request = account.client.build();

                        let query = request.query_email();
                        query.arguments().collapse_threads(false);
                        query.filter(jmap_client::core::query::Filter::and([
                            jmap_client::email::query::Filter::in_mailbox(mailbox_id).into(),
                            jmap_client::core::query::Filter::not([
                                jmap_client::email::query::Filter::From {
                                    value: account.address.clone(),
                                },
                            ]),
                        ]));

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

    pub fn get_mail(&self, selected_mailbox_idx: usize, selected_mail_idx: usize) -> Option<Email> {
        if let Some(mailboxes) = &self.mailboxes {
            let mailbox = &mailboxes[selected_mailbox_idx];

            if let Some(mails) = self.mails.get(mailbox.id().unwrap()) {
                if let Some(mails) = mails {
                    return Some(mails[selected_mail_idx].clone());
                }
            }

            debug!(
                "No mails available yet for mailbox '{}'",
                mailbox.name().unwrap_or_default()
            );
        } else {
            debug!("No mailboxes available yet");
        }

        None
    }

    pub fn select_next_mailbox(&mut self) {
        self.mailbox_list_state.select_next();
        // TODO: select first entry (if available)
        self.mail_list_state.select(None);
    }

    pub fn select_previous_mailbox(&mut self) {
        self.mailbox_list_state.select_previous();
        // TODO: select first entry (if available)
        self.mail_list_state.select(None);
    }

    pub fn select_mailbox(&mut self, idx: usize) {
        self.mailbox_list_state.select(Some(idx));
        // TODO: select first entry (if available)
        self.mail_list_state.select(None);
    }

    pub fn select_next_mail(&mut self) {
        if let Some(idx) = self.selected_mailbox_idx() {
            if let Some(mails) = self.get_mails(idx) {
                if !mails.is_empty() {
                    self.mail_list_state.next();
                    return;
                }
            }
        }

        self.mail_list_state.select(None);
    }

    pub fn select_previous_mail(&mut self) {
        if let Some(idx) = self.selected_mailbox_idx() {
            if let Some(mails) = self.get_mails(idx) {
                if !mails.is_empty() {
                    self.mail_list_state.previous();
                    return;
                }
            }
        }

        self.mail_list_state.select(None);
    }

    pub fn selected_mailbox_idx(&self) -> Option<usize> {
        self.mailbox_list_state.selected()
    }

    pub fn selected_mail_list_idx(&self) -> Option<usize> {
        self.mail_list_state.selected
    }

    pub fn get_mailbox_list_state_mut(&mut self) -> &mut widgets::ListState {
        &mut self.mailbox_list_state
    }

    pub fn get_mail_list_state_mut(&mut self) -> &mut tui_widget_list::ListState {
        &mut self.mail_list_state
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
