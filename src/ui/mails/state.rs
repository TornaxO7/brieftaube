use jmap_client::{client::Client, email::Email, mailbox::Mailbox};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::debug;

type MailboxId = String;

enum MailsInMailbox {
    NotInitialized,
    Fetching,
    Mails(Vec<Email>),
}

pub struct State {
    client: Arc<Client>,

    rx_mailboxes: mpsc::Receiver<Vec<Mailbox>>,

    rx_mails: mpsc::Receiver<Vec<Email>>,
    tx_mails: Arc<mpsc::Sender<Vec<Email>>>,

    mailboxes: Option<Vec<Mailbox>>,
    /// Contains all mails for each mailbox.
    mails: HashMap<MailboxId, Option<Vec<Email>>>,
}

impl State {
    pub fn new(client: Arc<Client>) -> Self {
        let (tx_mailboxes, rx_mailboxes) = mpsc::channel(1);
        let (tx_mails, rx_mails) = mpsc::channel(1);

        let client2 = client.clone();
        tokio::spawn(async move {
            let mut mailbox_response = {
                let mut request = client2.build();
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
            client,

            rx_mailboxes,

            rx_mails,
            tx_mails: Arc::new(tx_mails),

            mailboxes: None,
            mails: HashMap::new(),
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

    pub fn get_mails<'a>(&mut self, selected_mailbox_idx: usize) -> Option<Vec<String>> {
        if let Some(mailboxes) = &self.mailboxes {
            let selected_mailbox = mailboxes.get(selected_mailbox_idx).expect("Mailbox exists");
            let selected_mailbox_id = selected_mailbox.id().unwrap();

            if let Some(possible_mails) = self.mails.get(selected_mailbox_id) {
                if let Some(mails) = possible_mails {
                    return Some(
                        mails
                            .iter()
                            .map(|mail| mail.subject().unwrap().to_string())
                            .collect(),
                    );
                } else {
                    // We are still waiting for te mails to come in
                    match self.rx_mails.try_recv() {
                        Ok(mails) => {
                            let names = mails
                                .iter()
                                .map(|mail| mail.subject().unwrap_or("No subject").to_string())
                                .collect::<Vec<String>>();
                            self.mails
                                .insert(selected_mailbox_id.to_string(), Some(mails));
                            return Some(names);
                        }
                        Err(mpsc::error::TryRecvError::Empty) => {}
                        Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
                    }
                }
            } else {
                // indicate that fetching the mail started
                self.mails.insert(selected_mailbox_id.to_string(), None);

                let client2 = self.client.clone();
                let mailbox_id = selected_mailbox.id().unwrap().to_string();

                let tx = self.tx_mails.clone();
                tokio::spawn(async move {
                    // TODO: Error handling
                    let mut query_response = client2
                        .email_query(
                            Some(jmap_client::email::query::Filter::in_mailbox(mailbox_id)),
                            None::<Vec<_>>,
                        )
                        .await
                        .unwrap();

                    // TODO: Listen to changes
                    let ids = query_response.take_ids();

                    let mut mails = {
                        let mut request = client2.build();
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
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
