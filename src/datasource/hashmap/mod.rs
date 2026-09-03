mod mail;
mod mailbox;
mod root_mails;
mod thread;

use super::{BaseDataSource, types::GetState};
use crate::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxData,
    MailboxId, ThreadId,
};
use root_mails::RootMails;
use std::collections::HashMap;

#[derive(Default)]
pub struct HashMapDataSource {
    mails: HashMap<MailId, MailData>,
    mailboxes: HashMap<MailboxId, MailboxData>,
    threads: HashMap<ThreadId, Vec<MailId>>,
    root_mails: HashMap<MailboxId, RootMails>,

    mail_text_body: HashMap<MailId, MailDataTextBody>,
    mail_html_body: HashMap<MailId, MailDataHtmlBody>,
    mail_attachments: HashMap<MailId, Vec<MailDataAttachment>>,

    mail_get_state: Option<GetState>,
    mailboxes_get_state: Option<GetState>,
    threads_get_state: Option<GetState>,
}

impl HashMapDataSource {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BaseDataSource for HashMapDataSource {
    type Error = ();
}
