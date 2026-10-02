use crate::types::{MailDataCore, MailboxData};

#[derive(Debug)]
pub struct InitMailboxData {
    pub total_threads: usize,
    pub child_mailboxes: Vec<MailboxData>,
    pub first_mails: Vec<MailDataCore>,
}
