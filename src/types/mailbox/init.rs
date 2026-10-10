use crate::types::{CachedMail, CachedMailbox};

#[derive(Debug)]
pub struct InitMailboxData {
    pub total_threads: usize,
    pub child_mailboxes: Vec<CachedMailbox>,
    pub first_mails: Vec<CachedMail>,
}
