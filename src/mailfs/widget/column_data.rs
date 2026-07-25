use std::rc::Rc;

use crate::{
    backend::{
        Backend,
        mailbox::types::MailboxData,
        mails::types::{MailAddress, MailData, MailKeyword, MailPreview},
    },
    mailfs::state::{ColumnCtx, ColumnCtxEntry},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ColumnData<'a> {
    pub entries: Vec<ColumnEntry>,
    pub state: &'a mut TableState,
}

impl<'a> ColumnData<'a> {
    pub fn new(ctx: &'a mut ColumnCtx, backend: Rc<Backend>) -> Self {
        let entries = {
            let mut entries = Vec::new();

            for entry in ctx.entries.iter() {
                match entry {
                    ColumnCtxEntry::Mailbox(mailboxid) => {
                        let mailbox = backend.get_mailbox_data(mailboxid).unwrap();

                        let data = ColumnEntryData::from(mailbox.as_ref());
                        entries.push(ColumnEntry {
                            is_selected: false,
                            data,
                        });
                    }
                    ColumnCtxEntry::SingleMail(mail_id) => {
                        let mail = backend.get_mail_data(mail_id).unwrap();
                        let data = ColumnEntryData::single(&mail);
                        entries.push(ColumnEntry {
                            is_selected: false,
                            data,
                        });
                    }
                    ColumnCtxEntry::CollapsedThread(thread_id) => {
                        let thread = backend.get_thread_mail_ids(thread_id).unwrap();
                        let mail = backend.get_mail_data(&thread[0]).unwrap();
                        let data = ColumnEntryData::collapsed_thread(&mail);
                        entries.push(ColumnEntry {
                            is_selected: false,
                            data,
                        });
                    }
                    ColumnCtxEntry::UncollapsedThread(thread_id) => {
                        let thread = backend.get_thread_mail_ids(thread_id).unwrap();
                        let thread_len = thread.len();

                        let mails: Vec<MailData> = thread
                            .into_iter()
                            .map(|id| backend.get_mail_data(&id).unwrap())
                            .collect();

                        entries.push(ColumnEntry {
                            is_selected: false,
                            data: ColumnEntryData::thread_start(&mails[0]),
                        });

                        for mail in &mails[1..(thread_len - 1)] {
                            entries.push(ColumnEntry {
                                is_selected: false,
                                data: ColumnEntryData::thread_child(&mail),
                            });
                        }

                        entries.push(ColumnEntry {
                            is_selected: false,
                            data: ColumnEntryData::thread_end(&mails[thread_len - 1]),
                        });
                    }
                };
            }
            entries
        };

        Self {
            entries,
            state: &mut ctx.state,
        }
    }
}

#[derive(Debug)]
pub struct ColumnEntry {
    pub is_selected: bool,
    pub data: ColumnEntryData,
}

#[derive(Debug)]
pub enum ColumnEntryData {
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

impl ColumnEntryData {
    pub fn single(mail: &MailData) -> Self {
        Self::new(MailEntryType::Single, mail)
    }

    pub fn collapsed_thread(mail: &MailData) -> Self {
        Self::new(MailEntryType::ThreadCollapsed, mail)
    }

    pub fn thread_start(mail: &MailData) -> Self {
        Self::new(MailEntryType::ThreadStart, mail)
    }

    pub fn thread_child(mail: &MailData) -> Self {
        Self::new(MailEntryType::ThreadChild, mail)
    }

    pub fn thread_end(mail: &MailData) -> Self {
        Self::new(MailEntryType::ThreadEnd, mail)
    }

    fn new(ty: MailEntryType, mail: &MailData) -> Self {
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

impl From<&MailboxData> for ColumnEntryData {
    fn from(mailbox: &MailboxData) -> Self {
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
