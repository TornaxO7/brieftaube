use crate::backend::{Backend, MailboxNew, mailbox::error::MailboxValidationError};

impl Backend {
    pub fn create_mailbox(&self, mailbox: MailboxNew) -> Result<(), MailboxValidationError> {
        self.validate_mailbox(mailbox)?;
        todo!();
    }
}
