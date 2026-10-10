mod blob;
mod mail;
mod mailbox;
mod root_mails_linear;
mod thread;

use super::types::GetState;
use crate::{
    datasource::{Cache, types::QueryState},
    types::{BlobId, CachedMail, CachedMailbox, MailId, MailboxId, ThreadId},
};
use root_mails_linear::RootMails;
use std::collections::HashMap;

#[derive(Default)]
pub struct HashMapDataSource {
    mails: HashMap<MailId, CachedMail>,
    mailboxes: HashMap<MailboxId, CachedMailbox>,
    threads: HashMap<ThreadId, Vec<MailId>>,
    root_mails: HashMap<MailboxId, RootMails>,
    blobs: HashMap<BlobId, Vec<u8>>,

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

impl Cache for HashMapDataSource {}
