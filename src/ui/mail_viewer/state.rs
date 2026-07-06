use crate::{backend::Account, ui::MailId};
use chrono::DateTime;
use jmap_client::email::{Email, EmailAddress};
use ratatui::widgets::ScrollbarState;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct State {
    account: Arc<Account>,

    rx: mpsc::Receiver<Email>,
    tx: Arc<mpsc::Sender<Email>>,

    ctx: Option<Ctx>,
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        Self {
            rx,
            tx: Arc::new(tx),
            account,

            ctx: None,
        }
    }

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(mail) => self.ctx = Some(Ctx::new(mail)),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }
    }

    pub fn open_mail(&mut self, id: MailId) {
        self.ctx = None;
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
        self.ctx.as_mut().map(|ctx| ctx.scroll_down());
    }

    pub fn scroll_up(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_up());
    }

    pub fn scroll_left(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_left());
    }

    pub fn scroll_right(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_right());
    }

    pub fn get_render_data(&mut self) -> Option<RenderData<'_>> {
        self.ctx.as_mut().map(|ctx| ctx.render_data())
    }
}

#[derive(Debug)]
struct Ctx {
    pub mail: Email,
    /// String representation of mail
    pub mail_str: String,

    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,
}

impl Ctx {
    pub fn new(mail: Email) -> Self {
        let mail_str = Self::get_string_representation(&mail);

        let vertical = ScrollbarState::new(mail_str.lines().count());
        let horizontal =
            ScrollbarState::new(mail_str.lines().map(|line| line.len()).max().unwrap());

        Self {
            mail,
            mail_str,
            vertical,
            horizontal,
        }
    }

    pub fn render_data(&mut self) -> RenderData<'_> {
        RenderData {
            mail: &self.mail,
            mail_str: self.mail_str.as_str(),

            vertical: &mut self.vertical,
            horizontal: &mut self.horizontal,
        }
    }

    pub fn scroll_down(&mut self) {
        self.vertical.next();
    }

    pub fn scroll_up(&mut self) {
        self.vertical.prev();
    }

    pub fn scroll_right(&mut self) {
        self.horizontal.next();
    }

    pub fn scroll_left(&mut self) {
        self.horizontal.prev();
    }

    fn get_string_representation(mail: &Email) -> String {
        let date = DateTime::from_timestamp_millis(mail.received_at().unwrap())
            .unwrap()
            .format("%A, %d %B %Y %T");

        let from = Self::addresses_to_string(mail.from().unwrap());
        let to = Self::addresses_to_string(mail.to().unwrap());
        let subject = mail.subject().unwrap();

        let body = {
            let mut s = String::new();

            for body in mail.text_body().unwrap() {
                let id = body.part_id().unwrap();

                s.push_str(mail.body_value(id).unwrap().value());
            }

            s
        };

        format!(
            "\
Date: {date}
From: {from}
To: {to}
Subject: {subject}


{body}"
        )
    }

    fn addresses_to_string(addresses: &[EmailAddress]) -> String {
        let mut iter = addresses.iter();

        let mut s = format!("{}", iter.next().unwrap().email());

        for addr in iter {
            s.push_str(&format!(", {}", addr.email()))
        }

        s
    }
}

#[derive(Debug)]
pub struct RenderData<'a> {
    pub mail: &'a Email,
    pub mail_str: &'a str,
    pub vertical: &'a mut ScrollbarState,
    pub horizontal: &'a mut ScrollbarState,
}
