use crate::backend::{
    Backend, MailboxNew, MailboxValidate, ParentMailboxId, mailbox::error::MailboxValidationError,
    task_manager::TaskId,
};

impl Backend {
    pub fn create_mailbox(&self, mailbox: MailboxNew) -> Result<(), MailboxValidationError> {
        self.validate_mailbox(mailbox)?;

        self.task_manager.spawn(TaskId::MailboxSet, async move {
            todo!();
        });

        Ok(())
    }
}
