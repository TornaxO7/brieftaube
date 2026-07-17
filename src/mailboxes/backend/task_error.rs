use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error(transparent)]
    Client(#[from] jmap_client::Error),

    #[error(transparent)]
    CreateMailbox(#[from] ErrorCreateMailbox),
}

#[derive(Error, Debug)]
pub enum ErrorCreateMailbox {
    #[error(
        "Max mailbox depth reached for the mail server :( You can't create another sub-mailbox in the current mailbox. The maximum depth is {0} for the server."
    )]
    ReachedMaxDepth(usize),

    #[error("The mailbox name is too long. It can be at most {0} characters long.")]
    NameTooLong(usize),

    #[error("There's already a mailbox in the current mailbox with the name '{0}'.")]
    NameAlreadyUsed(String),
}
