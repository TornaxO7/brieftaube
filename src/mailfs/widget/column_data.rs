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

#[derive(Debug)]
pub struct ColumnDisplay<'a> {
    pub entries: Vec<ColumnDisplayEntry>,
    pub state: &'a mut TableState,
}

impl<'a> ColumnDisplay<'a> {
    pub fn new(column: Option<&'a mut ColumnState>, backend: Rc<Backend>) -> Option<Self> {
        let column = column?;

        let entries = column
            .entries()
            .iter()
            .map(|entry| match entry {
                ColumnStateEntry::Mailbox(id) => {
                    let mailbox = backend.mailbox_get_data(id)?;
                    Some(ColumnDisplayEntryData::mailbox(&mailbox))
                }
                ColumnStateEntry::SingleMail(mail_id) => {
                    let mail = backend.mail_get_data(mail_id)?;
                    Some(ColumnDisplayEntryData::mail(MailEntryType::Single, &mail))
                }
                ColumnStateEntry::CollapsedThread(mail_id, _) => {
                    let mail = backend.mail_get_data(mail_id)?;
                    Some(ColumnDisplayEntryData::mail(
                        MailEntryType::ThreadCollapsed,
                        &mail,
                    ))
                }
                ColumnStateEntry::ThreadStart(mail_id, _) => {
                    let mail = backend.mail_get_data(mail_id)?;
                    Some(ColumnDisplayEntryData::mail(
                        MailEntryType::ThreadStart,
                        &mail,
                    ))
                }
                ColumnStateEntry::ThreadChild(mail_id, _) => {
                    let mail = backend.mail_get_data(mail_id)?;
                    Some(ColumnDisplayEntryData::mail(
                        MailEntryType::ThreadChild,
                        &mail,
                    ))
                }
                ColumnStateEntry::ThreadEnd(mail_id, _) => {
                    let mail = backend.mail_get_data(mail_id)?;
                    Some(ColumnDisplayEntryData::mail(
                        MailEntryType::ThreadEnd,
                        &mail,
                    ))
                }
            })
            .map(|data| {
                data.map(|data| ColumnDisplayEntry {
                    is_selected: false,
                    data,
                })
            })
            .collect::<Option<Vec<ColumnDisplayEntry>>>()?;

        Some(Self {
            entries,
            state: &mut column.state,
        })
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

#[derive(Debug)]
pub enum RightColumn<'a> {
    ColumnData(ColumnDisplay<'a>),
    MailPreview(MailPreview),
}

fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
