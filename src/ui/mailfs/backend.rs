use crate::backend::types::{AccountId, MailId, MailboxId};
use std::thread::ThreadId;

#[derive(Debug, Clone)]
pub enum MailfsMessage {
    LoadMailbox {
        mailbox_id: MailboxId,
        offset: usize,
        limit: usize,
    },
}

#[derive(Debug, Clone)]
pub struct MailfsAccount {
    pub is_primary: bool,
    pub id: AccountId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MailfsAccountEntry {
    pub name: String,
    pub accounts: Vec<MailfsAccount>,
}

#[derive(Debug, Clone)]
pub struct MailfsMailAttachment {
    pub name: String,
    pub size: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub enum MailfsMailboxEntry {
    Mailbox {
        id: MailboxId,
        name: String,
        unread_mails: usize,
    },
    Mail {
        id: MailId,
        thread_id: ThreadId,
        keywords: String,
        from: String,
        to: String,
        cc: String,
        subject: String,
        preview: String,
        received_at: String,
        has_attachment: bool,
        attachments: Vec<MailfsMailAttachment>,
    },
}

#[derive(Debug, Clone)]
pub enum MailfsColumn {
    Accounts(Vec<MailfsAccountEntry>),
    Mailbox(Vec<MailfsMailboxEntry>),
}

#[derive(Debug, Clone)]
pub struct MailfsSnapshot {
    pub left: MailfsColumn,
    pub center: MailfsColumn,
    pub right: MailfsColumn,
}
