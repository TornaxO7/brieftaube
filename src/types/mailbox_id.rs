pub const TOP_PARENT_MAILBOX_ID: ParentMailboxId = None;
pub type ParentMailboxId = Option<MailboxId>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailboxId(pub String);

impl MailboxId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<MailboxId> for String {
    fn from(id: MailboxId) -> Self {
        id.0
    }
}

impl From<&MailboxId> for String {
    fn from(id: &MailboxId) -> Self {
        id.0.clone()
    }
}
