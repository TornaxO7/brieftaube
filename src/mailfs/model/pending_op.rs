use crate::backend::{MailId, MailboxId, ParentMailboxId, ThreadId};

/// Tasks which require data from the backend which may not be there yet.
#[derive(Debug)]
pub enum PendingOp {
    InitMailbox(ParentMailboxId),
    UncollapseThread(OpUncollapseThread),
    MailAttachments(MailId),
    MoveMailboxUp(OpMoveMailboxUp),
}

#[derive(Debug)]
pub struct OpUncollapseThread {
    // in which mailbox the thread is
    pub column_mailbox: MailboxId,
    pub collapsed_mail_id: MailId,
    pub thread_id: ThreadId,
}

#[derive(Debug)]
pub struct OpMoveMailboxUp {
    pub parent: ParentMailboxId,
    pub mailbox: MailboxId,
}
