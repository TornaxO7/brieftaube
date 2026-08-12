use crate::backend::{MailId, MailboxId, ParentMailboxId, ThreadId};

/// Tasks which require data from the backend which may not be there yet.
#[derive(Debug)]
pub enum PendingOp {
    InitMailbox(OpInitMailbox),
    UncollapseThread(OpUncollapseThread),
    PreviewMail(OpPreviewMail),
}

#[derive(Debug)]
pub struct OpInitMailbox(pub ParentMailboxId);

#[derive(Debug)]
pub struct OpUncollapseThread {
    // in which mailbox the thread is
    pub column_mailbox: MailboxId,
    pub collapsed_mail_id: MailId,
    pub thread_id: ThreadId,
}

#[derive(Debug)]
pub struct OpPreviewMail(pub MailId);
