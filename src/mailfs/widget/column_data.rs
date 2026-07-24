use ratatui::widgets::TableState;

use crate::backend::{
    mailbox::types::MailboxData,
    mails::types::{MailAddress, MailPreview},
};

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
        ty: MailEntryType,

        from: String,
        subject: &'a str,
        received_at: &'a str,
        has_attachment: bool,
        is_unread: bool,
    },
}

impl<'a> From<&'a MailboxData> for ColumnEntryData<'a> {
    fn from(mailbox: &'a MailboxData) -> Self {
        Self::Mailbox {
            name: &mailbox.name,
            unread_mails: mailbox.unread_mails,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailEntryType {
    Single,
    ThreadRoot,
    ThreadChild,
}

pub enum RightColumn<'a> {
    ColumnData(Option<ColumnData<'a>>),
    MailPreview(Option<MailPreview>),
}

fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
