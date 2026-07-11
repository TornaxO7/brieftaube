mod root_mails;
mod threads;

use std::{collections::HashMap, thread::ThreadId};

use crate::backend::mailboxes::mailbox::{root_mails::RootMails, threads::ThreadCtx};
use jmap_client::{client::Client, email::Email, mailbox::Mailbox};

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

    pub fn root_mails(&self) -> Option<&[Email]> {
        self.root_mails.as_ref().map(|mails| mails.mails())
    }

    pub async fn get_or_fetch_root_mails(&mut self, client: &Client) -> &[Email] {
        self.root_mails
            .get_or_insert(RootMails::new(client, self.id().to_string()).await)
            .mails()
    }

    pub fn mailbox(&self) -> &Mailbox {
        &self.inner
    }
}
