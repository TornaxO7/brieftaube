use crate::{
    backend::{
        Backend,
        mailbox::types::MailboxData,
        mails::types::{MailAddress, MailData, MailKeyword},
    },
    mailfs::{
        state::{ColumnState, ColumnStateEntry, ColumnStateEntryType},
        widget::{error, mail_preview::MailPreview, selection_type::SelectionType},
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
    pub fn new(
        column: Option<&'a mut ColumnState>,
        backend: Rc<Backend>,
    ) -> Result<Self, error::DataNotAvaiableYet> {
        let column = column.ok_or(error::DataNotAvaiableYet)?;

        let entries = column
            .entries()
            .iter()
            .map(|entry| ColumnDisplayEntry::try_from((entry, backend.clone())))
            .collect::<Result<Vec<ColumnDisplayEntry>, error::DataNotAvaiableYet>>()?;

        Ok(Self {
            entries,
            state: &mut column.state,
        })
    }
}

#[derive(Debug)]
pub struct ColumnDisplayEntry {
    pub selection_type: Option<SelectionType>,
    pub data: ColumnDisplayEntryData,
}

impl TryFrom<(&ColumnStateEntry, Rc<Backend>)> for ColumnDisplayEntry {
    type Error = error::DataNotAvaiableYet;

    fn try_from((entry, backend): (&ColumnStateEntry, Rc<Backend>)) -> Result<Self, Self::Error> {
        let selection_type = entry.selection.map(SelectionType::from);
        let data = match &entry.entry_type {
            ColumnStateEntryType::Mailbox(id) => {
                let mailbox = backend.mailbox_get_data(id).unwrap();
                ColumnDisplayEntryData::mailbox(&mailbox)
            }
            ColumnStateEntryType::SingleMail(mail_id) => {
                let mail = backend
                    .mail_get_data(mail_id)
                    .ok_or(error::DataNotAvaiableYet)?;
                ColumnDisplayEntryData::mail(MailEntryType::Single, &mail)
            }
            ColumnStateEntryType::CollapsedThread(mail_id, _) => {
                let mail = backend
                    .mail_get_data(mail_id)
                    .ok_or(error::DataNotAvaiableYet)?;
                ColumnDisplayEntryData::mail(MailEntryType::ThreadCollapsed, &mail)
            }
            ColumnStateEntryType::ThreadStart { mail_id, .. } => {
                let mail = backend
                    .mail_get_data(mail_id)
                    .ok_or(error::DataNotAvaiableYet)?;
                ColumnDisplayEntryData::mail(MailEntryType::ThreadStart, &mail)
            }
            ColumnStateEntryType::ThreadChild(mail_id, _) => {
                let mail = backend
                    .mail_get_data(mail_id)
                    .ok_or(error::DataNotAvaiableYet)?;
                ColumnDisplayEntryData::mail(MailEntryType::ThreadChild, &mail)
            }
            ColumnStateEntryType::ThreadEnd(mail_id, _) => {
                let mail = backend
                    .mail_get_data(mail_id)
                    .ok_or(error::DataNotAvaiableYet)?;
                ColumnDisplayEntryData::mail(MailEntryType::ThreadEnd, &mail)
            }
        };

        Ok(Self {
            selection_type,
            data,
        })
    }
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
    pub fn mailbox(mailbox: &MailboxData) -> Self {
        Self::Mailbox {
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
