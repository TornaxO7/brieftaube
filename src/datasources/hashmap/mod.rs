mod mail;
mod mailbox;
mod thread;
mod utils;

use crate::{
    datasources::{
        BaseDataSource,
        types::{GetState, QueryState},
    },
    types::{
        MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxData,
        MailboxId, ThreadId,
    },
};
use std::{collections::HashMap, sync::RwLock};
use utils::root_mails::RootMails;

pub struct HashMapDataSource {
    inner: RwLock<Inner>,
}

impl HashMapDataSource {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Inner::default()),
        }
    }
}

#[derive(Default)]
struct Inner {
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

impl BaseDataSource for HashMapDataSource {
    type Error = ();

    fn mail_get_state(&self) -> Option<GetState> {
        let inner = self.inner.read().unwrap();
        inner.mail_get_state.clone()
    }

    fn mailboxes_get_state(&self) -> Option<GetState> {
        let inner = self.inner.read().unwrap();
        inner.mailboxes_get_state.clone()
    }

    fn threads_get_state(&self) -> Option<GetState> {
        let inner = self.inner.read().unwrap();
        inner.threads_get_state.clone()
    }

    fn root_mails_query_state(&self, id: &MailboxId) -> Option<QueryState> {
        let inner = self.inner.read().unwrap();
        inner
            .root_mails
            .get(id)
            .map(|root_mails| root_mails.state().clone())
    }
}
