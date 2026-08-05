use crate::backend::{mailbox, mails, threads};

pub struct Store {
    pub mails: mails::Store,
    pub mailbox: mailbox::Store,
    pub threads: threads::Store,
}

impl Store {
    pub fn new() -> Self {
        Self {
            mails: mails::Store::new(),
            mailbox: mailbox::Store::new(),
            threads: threads::Store::new(),
        }
    }
}
