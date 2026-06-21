use jmap_client::{client::Client, email::Email, mailbox::Mailbox};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

type MailboxId = String;

pub struct State {
    client: Arc<Client>,

    rx_mailboxes: mpsc::Receiver<Vec<Mailbox>>,

    mailboxes: Option<Vec<Mailbox>>,
    mails: HashMap<MailboxId, Vec<Email>>,

    mailbox_names: Option<Vec<String>>,
}

impl State {
    pub fn new(client: Arc<Client>) -> Self {
        let (tx_mailboxes, rx_mailboxes) = mpsc::channel(1);

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

            mailboxes: None,
            mails: HashMap::new(),

            mailbox_names: None,
        }
    }

    pub fn get_mailbox_names(&mut self) -> Option<&Vec<String>> {
        if self.mailboxes.is_none() {
            match self.rx_mailboxes.try_recv() {
                Ok(mailboxes) => {
                    self.mailbox_names = Some(
                        mailboxes
                            .iter()
                            .map(|mailbox| mailbox.name().unwrap().to_string())
                            .collect::<Vec<String>>(),
                    );
                    self.mailboxes = Some(mailboxes);
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
            }
        }

        self.mailbox_names.as_ref()
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
