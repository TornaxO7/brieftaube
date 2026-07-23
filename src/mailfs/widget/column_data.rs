use crate::backend::{
    mailbox::types::MailboxId,
    mails::types::{MailId, ThreadId},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ColumnData<'a> {
    pub entries: Vec<ColumnEntry<'a>>,
    pub state: &'a mut TableState,
}

#[derive(Debug)]
pub struct ColumnEntry<'a> {
    pub is_selected: bool,
    pub data: ColumnEntryData<'a>,
}

#[derive(Debug)]
pub enum ColumnEntryData<'a> {
    Mailbox {
        name: &'a str,
        unread_mails: usize,
    },
    Mail {
        thread: ThreadId,
        ty: MailEntryType,

        from: &'a str,
        subject: &'a str,
        received_at: &'a str,
        has_attachment: bool,
        is_unread: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailEntryType {
    Single,
    ThreadRoot,
    ThreadChild,
}
