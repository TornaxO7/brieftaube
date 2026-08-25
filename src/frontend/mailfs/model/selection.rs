use crate::backend::ParentMailboxId;

#[derive(Debug, Clone)]
pub struct Selection {
    pub mailbox: ParentMailboxId,
    pub ty: SelectionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Selected,
    Cut,
}
