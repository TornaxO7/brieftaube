use crate::backend::{
    mailbox::types::MailboxId,
    mails::types::{MailId, ThreadId},
};
use ratatui::widgets::TableState;

pub struct ColumnData<'a> {
    pub entries: Vec<ColumnEntry<'a>>,
    pub state: &'a mut TableState,
}

pub struct ColumnEntry<'a> {
    pub is_selected: bool,
    pub data: ColumnEntryData<'a>,
}

#[derive(Debug)]
pub enum ColumnEntryData<'a> {
    Mailbox {
        id: MailboxId,
        name: &'a str,
        unread_mails: usize,
    },
    Mail {
        id: MailId,
        thread: ThreadId,
        ty: MailEntryType,

        from: &'a str,
        subject: &'a str,
        received_at: &'a str,
        has_attachment: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailEntryType {
    Single,
    ThreadRoot,
    ThreadChild,
}
