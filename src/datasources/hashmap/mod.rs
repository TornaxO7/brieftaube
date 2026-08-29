mod mail;
mod mailbox;
mod thread;
mod utils;

use crate::{
    datasources::{
        BaseDataSource,
        types::{GetState, QueryState},
    },
    types::{MailData, MailId, MailboxData, MailboxId, ThreadId},
};
use mail::QueryError;
use std::{collections::HashMap, sync::RwLock};
use utils::RootMails;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    QueryError(#[from] mail::QueryError),
}

pub struct HashMapDataSource {
    inner: RwLock<Inner>,
}

struct Inner {
    mails: HashMap<MailId, MailData>,
    mailboxes: HashMap<MailboxId, MailboxData>,
    threads: HashMap<ThreadId, Vec<MailId>>,
    root_mails: HashMap<MailboxId, RootMails>,

    mail_get_state: Option<GetState>,
    mailboxes_get_state: Option<GetState>,
    threads_get_state: Option<GetState>,
}

impl BaseDataSource for HashMapDataSource {
    type Error = Error;
}
