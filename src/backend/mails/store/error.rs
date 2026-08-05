use super::ThreadId;

#[derive(Debug, Clone)]
pub enum UnfoldError {
    MissingThreadMails(ThreadId),
    MailboxMailsMissing,
}
