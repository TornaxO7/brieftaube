use crate::{
    backend::{MailId, ParentMailboxId, TOP_PARENT_MAILBOX_ID},
    mailfs::model::ColumnState,
};
use std::collections::HashMap;

pub enum RightColumn {
    Column(ParentMailboxId),
    Preview(MailId),
}

pub struct Columns {
    columns: HashMap<ParentMailboxId, ColumnState>,
    pub right_column: Option<RightColumn>,
}

impl Columns {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            right_column: None,
        }
    }

    pub fn get(&self, id: &ParentMailboxId) -> Option<&ColumnState> {
        self.columns.get(id)
    }

    pub fn get_mut(&mut self, id: &ParentMailboxId) -> Option<&mut ColumnState> {
        self.columns.get_mut(id)
    }
}
