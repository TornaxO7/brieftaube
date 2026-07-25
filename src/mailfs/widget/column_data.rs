use std::rc::Rc;

use crate::{
    backend::{
        Backend,
        mailbox::types::MailboxData,
        mails::types::{MailAddress, MailPreview},
    },
    mailfs::state::{ColumnCtx, ColumnCtxEntry},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ColumnData<'a> {
    pub entries: Vec<ColumnEntry<'a>>,
    pub state: &'a mut TableState,
}

impl<'a> ColumnData<'a> {
    pub fn new(ctx: &'a mut ColumnCtx, backend: Rc<Backend>) -> Self {
        let entries = ctx
            .entries
            .iter()
            .map(|entry| {
                // TODO: HERE
                let data = match entry {
                    ColumnCtxEntry::Mailbox(mailboxid) => {
                        let mailbox = backend.get_mailbox_data(mailboxid).unwrap();
                        todo!();
                    }
                    ColumnCtxEntry::SingleMail(mail_id) => todo!(),
                    ColumnCtxEntry::CollapsedThread(thread_id) => todo!(),
                    ColumnCtxEntry::UncollapsedThread(thread_id) => todo!(),
                };

                ColumnEntry {
                    is_selected: false,
                    data,
                }
            })
            .collect();

        Self {
            entries,
            state: &mut ctx.state,
        }
    }
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

impl<'a> RightColumn<'a> {
    pub fn new(ctx: Option<&'a mut ColumnCtx>) -> Self {
        todo!()
    }
}

fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
