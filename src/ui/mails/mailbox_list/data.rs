use jmap_client::{client::Client, mailbox::Mailbox};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

pub struct Data {
    client: Arc<Client>,

    mailbox_names: Option<Vec<String>>,
    mailboxes: Option<Vec<Mailbox>>,
    rx: mpsc::Receiver<Vec<Mailbox>>,
}

impl Data {
    pub fn new(client: Arc<Client>) -> Self {
        let (tx, rx) = mpsc::channel(8);

        let c = client.clone();
        tokio::spawn(async move {
            let mut request = c.build();
            request.get_mailbox().ids::<[_; 0], String>(None);

            let mut res = request.send_get_mailbox().await.unwrap();
            debug!("Got mailbox response: {:#?}", res);
            tx.send(res.take_list()).await.unwrap();
        });

        Self {
            client,
            mailboxes: None,
            mailbox_names: None,
            rx,
        }
    }

    pub fn get_mailbox_names(&mut self) -> Option<&Vec<String>> {
        if self.mailboxes.is_none() {
            match self.rx.try_recv() {
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

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").finish()
    }
}
