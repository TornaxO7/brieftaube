use crate::backend::{mails::types::MailId, threads::types::ThreadId};

#[derive(Debug, Clone)]
pub enum CollapsedMail {
    SingleMail(MailId),
    CollapsedThread(ThreadId),
}
