use crate::{backend::Account, ui::MailId};
use chrono::DateTime;
use jmap_client::email::{Email, EmailAddress};
use ratatui::widgets::ScrollbarState;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct State {}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        Self {}
    }

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(mail) => self.ctx = Some(Ctx::new(mail)),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }
    }

    pub fn open_mail(&mut self, id: MailId) {}
}
