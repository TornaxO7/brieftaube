mod mail;
mod mailbox;
mod root_mails_linear;
mod thread;

use super::{BaseDataSource, types::GetState};
use crate::{
    datasource::types::QueryState,
    types::{
        MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody, MailId, MailboxData,
        MailboxId, ThreadId,
    },
};
use root_mails_linear::RootMails;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub struct HashMapDataSourceError;

impl std::fmt::Display for HashMapDataSourceError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!("This should never happen...")
    }
}

#[derive(Default)]
pub struct HashMapDataSource {
    mails_core: HashMap<MailId, MailDataCore>,
    mails_preview: HashMap<MailId, MailDataPreview>,
    mail_text_body: HashMap<MailId, MailDataTextBody>,
    mail_html_body: HashMap<MailId, MailDataHtmlBody>,

    mailboxes: HashMap<MailboxId, MailboxData>,
    threads: HashMap<ThreadId, Vec<MailId>>,
    root_mails: HashMap<MailboxId, RootMails>,

    root_mails_state: HashMap<MailboxId, QueryState>,
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
    type Error = HashMapDataSourceError;
}
