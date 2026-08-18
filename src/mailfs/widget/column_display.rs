use crate::{
    backend::{
        Backend,
        mailbox::types::MailboxData,
        mails::types::{MailAddress, MailData, MailKeyword},
    },
    mailfs::{
        model::{ColumnEntry, ColumnState, EntryId, SelectionType},
        widget::selection_type::DisplaySelectionType,
    },
};
use ratatui::widgets::TableState;
use std::{collections::HashMap, sync::Arc};
use throbber_widgets_tui::ThrobberState;

#[derive(Debug)]
pub enum ColumnDisplay<'a> {
    Loading(&'a mut ThrobberState),
    Loaded {
        entries: Vec<ColumnDisplayEntry>,
        state: &'a mut TableState,
    },
}

impl<'a> ColumnDisplay<'a> {
    pub fn new(
        state: &'a mut ColumnState,
        selection_mapping: &HashMap<EntryId, SelectionType>,
        backend: Arc<Backend>,
    ) -> Self {
        match state {
            ColumnState::Loading(throbber) => Self::Loading(throbber),
            ColumnState::Loaded(column) => {
                let entries: Vec<ColumnDisplayEntry> = column
                    .entries()
                    .iter()
                    .map(|entry| {
                        let id = EntryId::from(entry);
                        let selection = selection_mapping.get(&id).map(DisplaySelectionType::from);

                        ColumnDisplayEntry::new(entry, selection, backend.clone())
                    })
                    .collect();

                Self::Loaded {
                    entries,
                    state: &mut column.state,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ColumnDisplayEntry {
    pub selection_type: Option<DisplaySelectionType>,
    pub data: ColumnDisplayEntryData,
}

impl ColumnDisplayEntry {
    pub fn new(
        entry: &ColumnEntry,
        selection_type: Option<DisplaySelectionType>,
        backend: Arc<Backend>,
    ) -> Self {
        let data = match entry {
            ColumnEntry::Mailbox(id) => {
                let mailbox = backend.get_mailbox_data(id).unwrap();
                ColumnDisplayEntryData::mailbox(&mailbox)
            }
            ColumnEntry::SingleMail(mail_id) => {
                let mail = backend.get_mail(mail_id).unwrap();
                ColumnDisplayEntryData::mail(MailEntryType::Single, &mail)
            }
            ColumnEntry::CollapsedThread(mail_id, _) => {
                let mail = backend.get_mail(mail_id).unwrap();
                ColumnDisplayEntryData::mail(MailEntryType::ThreadCollapsed, &mail)
            }
            ColumnEntry::ThreadStart { mail_id, .. } => {
                let mail = backend.get_mail(mail_id).unwrap();
                ColumnDisplayEntryData::mail(MailEntryType::ThreadStart, &mail)
            }
            ColumnEntry::ThreadChild(mail_id, _) => {
                let mail = backend.get_mail(mail_id).unwrap();
                ColumnDisplayEntryData::mail(MailEntryType::ThreadChild, &mail)
            }
            ColumnEntry::ThreadEnd(mail_id, _) => {
                let mail = backend.get_mail(mail_id).unwrap();
                ColumnDisplayEntryData::mail(MailEntryType::ThreadEnd, &mail)
            }
        };

        Self {
            data,
            selection_type,
        }
    }
}

#[derive(Debug)]
pub enum ColumnDisplayEntryData {
    Mailbox {
        name: String,
        unread_mails: usize,
        sort_order: u32,
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
    pub fn mailbox(mailbox: &MailboxData) -> Self {
        Self::Mailbox {
            sort_order: mailbox.sort_order,
            name: mailbox.name.clone(),
            unread_mails: mailbox.unread_mails,
        }
    }

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailEntryType {
    Single,
    ThreadCollapsed,
    ThreadStart,
    ThreadChild,
    ThreadEnd,
}

fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
