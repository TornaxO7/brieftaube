use crate::{
    backend::{
        Backend,
        mailbox::types::MailboxData,
        mails::types::{MailAddress, MailData, MailKeyword},
    },
    mailfs::{
        state::{ColumnState, ColumnStateEntry},
        widget::mail_preview::MailPreview,
    },
};
use ratatui::widgets::TableState;
use std::rc::Rc;
use throbber_widgets_tui::ThrobberState;

#[derive(Debug)]
pub enum ColumnDisplay<'a> {
    Loading {
        state: &'a mut ThrobberState,
    },
    Loaded {
        entries: Vec<ColumnDisplayEntry>,
        state: &'a mut TableState,
    },
}

impl<'a> ColumnDisplay<'a> {
    pub fn new(column: &'a mut ColumnState, backend: Rc<Backend>) -> Self {
        match column {
            ColumnState::Loading { state } => Self::Loading { state },
            ColumnState::Loaded { entries, state, .. } => {
                let entries = entries
                    .into_iter()
                    .map(|entry| match entry {
                        ColumnStateEntry::Mailbox(id) => {
                            let mailbox = backend.mailbox_get_data(id).unwrap();
                            ColumnDisplayEntryData::mailbox(&mailbox)
                        }
                        ColumnStateEntry::SingleMail(id) => {
                            let mail = backend.mail_get_data(&id).unwrap();
                            ColumnDisplayEntryData::mail(MailEntryType::Single, &mail)
                        }
                        ColumnStateEntry::CollapsedThread(mail_id, _) => {
                            let mail = backend.mail_get_data(&mail_id).unwrap();
                            ColumnDisplayEntryData::mail(MailEntryType::ThreadCollapsed, &mail)
                        }
                        ColumnStateEntry::ThreadStart(mail_id, _) => {
                            let mail = backend.mail_get_data(&mail_id).unwrap();
                            ColumnDisplayEntryData::mail(MailEntryType::ThreadStart, &mail)
                        }
                        ColumnStateEntry::ThreadChild(mail_id, _) => {
                            let mail = backend.mail_get_data(&mail_id).unwrap();
                            ColumnDisplayEntryData::mail(MailEntryType::ThreadChild, &mail)
                        }
                        ColumnStateEntry::ThreadEnd(mail_id, _) => {
                            let mail = backend.mail_get_data(&mail_id).unwrap();
                            ColumnDisplayEntryData::mail(MailEntryType::ThreadEnd, &mail)
                        }
                    })
                    .map(|data| ColumnDisplayEntry {
                        is_selected: false,
                        data,
                    })
                    .collect();

                Self::Loaded { entries, state }
            }
        }
    }
}

#[derive(Debug)]
pub struct ColumnDisplayEntry {
    pub is_selected: bool,
    pub data: ColumnDisplayEntryData,
}

#[derive(Debug)]
pub enum ColumnDisplayEntryData {
    Mailbox {
        name: String,
        unread_mails: usize,
    },
    Mail {
        ty: MailEntryType,

        from: String,
        subject: String,
        received_at: String,
        has_attachment: bool,
        is_unread: bool,
    },
}

impl ColumnDisplayEntryData {
    pub fn mail(ty: MailEntryType, mail: &MailData) -> Self {
        Self::Mail {
            ty,
            from: addresses_to_string(&mail.from),
            subject: mail.subject.clone(),
            received_at: mail
                .received_at
                .format("%a, %e %b %Y, %H:%M:%S")
                .to_string(),
            has_attachment: mail.has_attachment,
            is_unread: !mail.keywords.contains(&MailKeyword::Seen),
        }
    }

    pub fn mailbox(mailbox: &MailboxData) -> Self {
        Self::Mailbox {
            name: mailbox.name.clone(),
            unread_mails: mailbox.unread_mails,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailEntryType {
    Single,
    ThreadCollapsed,
    ThreadStart,
    ThreadChild,
    ThreadEnd,
}

pub enum RightColumn<'a> {
    ColumnData(ColumnDisplay<'a>),
    MailPreview(MailPreview<'a>),
}

fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
