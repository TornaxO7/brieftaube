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
    pub fn new(account: Arc<Account>) -> Self {}

    pub fn reset(&mut self) {}

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(sent_mailbox_id) => self.draft_mailbox_id = Some(sent_mailbox_id),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }
    }
}

// TODO: Create directory which the user can use to just copy the attachment files to
