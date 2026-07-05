use crate::{backend::Account, ui::MailboxId};
use chrono::Local;
use jmap_client::email::{EmailBodyPart, EmailBodyValue};
use mail_parser::MessageParser;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct State {
    account: Arc<Account>,

    raw_mail: String,

    rx: mpsc::Receiver<MailboxId>,
    _tx: Arc<mpsc::Sender<MailboxId>>,
    draft_mailbox_id: Option<MailboxId>,

    scroll_offset: (u16, u16),
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let acc = account.clone();
        let tx = Arc::new(tx);
        let tx2 = tx.clone();

        tokio::spawn(async move {
            let mut request = acc.client.build();
            request.get_mailbox().ids(None::<[String; 0]>);
            let response = request.send_get_mailbox().await.unwrap();

            let sent_mailbox = response
                .list()
                .iter()
                .find(|mailbox| mailbox.role() == jmap_client::mailbox::Role::Drafts)
                .unwrap();

            tx2.send(sent_mailbox.id().unwrap().to_string())
                .await
                .unwrap();
        });

        let mut state = Self {
            account: account.clone(),
            raw_mail: String::new(),
            scroll_offset: (0, 0),
            draft_mailbox_id: None,

            rx,
            _tx: tx,
        };

        state.reset();
        state
    }

    pub fn reset(&mut self) {
        let address = self.account.address.clone();

        self.raw_mail = format!(
            "\
From: {address}
To:
Subject:

"
        );
    }

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(sent_mailbox_id) => self.draft_mailbox_id = Some(sent_mailbox_id),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset.0 = self.scroll_offset.0.saturating_sub(1);
    }

    pub fn get_mail(&self) -> &str {
        self.raw_mail.as_str()
    }

    pub fn open_mail_in_editor(&mut self) {
        let tmp_file = get_tmp_file();

        std::fs::write(&tmp_file, &self.raw_mail).unwrap();
        std::process::Command::new("hx")
            .arg(&tmp_file)
            .output()
            .unwrap();

        // TODO: Check if the changed mail is correct
        self.raw_mail = std::fs::read_to_string(&tmp_file).unwrap();
    }

    pub fn send_mail(&mut self) {
        if let Some(draft_id) = self.draft_mailbox_id.clone() {
            let account = self.account.clone();
            let raw_mail = self.raw_mail.clone();

            tokio::spawn(async move {
                let parsed = MessageParser::new().parse(raw_mail.as_bytes()).unwrap();

                let mut request = account.client.build();

                request
                    .set_email()
                    .create()
                    .sent_at(Local::now().timestamp())
                    .from(mail_parser_address_to_jmap_client_address(
                        parsed.from().unwrap(),
                    ))
                    .to(mail_parser_address_to_jmap_client_address(
                        parsed.to().unwrap(),
                    ))
                    .subject(parsed.subject().unwrap())
                    .mailbox_id(&draft_id, true)
                    .body_value(
                        "1".to_string(),
                        EmailBodyValue::from(parsed.body_text(0).unwrap().to_string()),
                    )
                    .text_body(EmailBodyPart::new().part_id("1").content_type("text/plain"));

                request.send_set_email().await.unwrap();
            });
        }
    }
}

// TODO: Create directory which the user can use to just copy the attachment files to
fn get_tmp_file() -> PathBuf {
    let xdg = crate::get_xdg();

    xdg.place_cache_file("temp.mail").unwrap()
}

fn mail_parser_address_to_jmap_client_address(
    addr: &mail_parser::Address,
) -> Vec<jmap_client::email::EmailAddress> {
    match addr {
        mail_parser::Address::List(list) => list
            .iter()
            .map(|addr| match addr.name() {
                Some(name) => {
                    jmap_client::email::EmailAddress::from((name, addr.address().unwrap()))
                }
                None => jmap_client::email::EmailAddress::from(addr.address().unwrap()),
            })
            .collect(),
        mail_parser::Address::Group(_group) => todo!(),
    }
}
