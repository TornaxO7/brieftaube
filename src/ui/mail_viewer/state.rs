use crate::{backend::Account, ui::MailId};
use jmap_client::email::Email;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct State {
    account: Arc<Account>,

    rx: mpsc::Receiver<Email>,
    tx: Arc<mpsc::Sender<Email>>,
    mail: Option<Email>,
    scroll_offset: (u16, u16),
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        Self {
            rx,
            tx: Arc::new(tx),
            account,
            mail: None,
            scroll_offset: (0, 0),
        }
    }

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(mail) => self.mail = Some(mail),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }
    }

    pub fn open_mail(&mut self, id: MailId) {
        let account = self.account.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let client = &account.client;

            let mut response = {
                let mut request = client.build();
                request
                    .get_email()
                    .ids(Some([id]))
                    .arguments()
                    .fetch_text_body_values(true);
                request.send_get_email().await.unwrap()
            };

            tx.send(response.take_list()[0].clone()).await.unwrap();
        });
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset.0 = self.scroll_offset.0.saturating_sub(1);
    }

    pub fn get_mail(&self) -> Option<&Email> {
        self.mail.as_ref()
    }
}
