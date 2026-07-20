use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailboxValidationError {
    #[error("Backend is not initialised yet")]
    NotInitialised,
    #[error("No name given for the new mailbox")]
    MissingName,
    #[error("Mailbox would exceed the server's max depth of {max}")]
    MaxDepthExceeded { max: usize },
    #[error("Mailbox name exceeds the server's max length of {max} octets")]
    NameTooLong { max: usize },
    #[error("A mailbox named '{name}' already exists in the target mailbox")]
    DuplicateName { name: String },
}
