mod root_mails;
mod threads;

use crate::backend::mailboxes::mailbox::{root_mails::RootMails, threads::ThreadCtx};
use jmap_client::mailbox::Mailbox;
use std::{collections::HashMap, thread::ThreadId};

pub struct MailboxCtx {
    inner: Mailbox,

    root_mails: Option<RootMails>,
    threads: HashMap<ThreadId, ThreadCtx>,
}

impl MailboxCtx {
    pub fn new(inner: Mailbox) -> Self {
        Self {
            inner,
            root_mails: None,
            threads: HashMap::default(),
        }
    }

    pub fn id(&self) -> &str {
        self.inner.id().unwrap()
    }

    pub fn mailbox(&self) -> &Mailbox {
        &self.inner
    }
}
