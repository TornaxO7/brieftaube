use crate::backend::{MailId, ParentMailboxId};

pub enum RightColumn {
    Column(ParentMailboxId),
    Preview(MailId),
}
