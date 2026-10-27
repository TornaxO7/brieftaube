use crate::types::INTERN_ID_PREFIX;

pub const ROOT_MAILBOX_ID: ParentMailboxId = None;
pub type ParentMailboxId = Option<MailboxId>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailboxId(pub String);

impl MailboxId {
    pub fn new_intern() -> Self {
        let suffix: String = std::iter::repeat_with(fastrand::alphanumeric)
            .take(10)
            .collect();

        Self(format!("{}{}", INTERN_ID_PREFIX, suffix))
    }

    pub fn is_intern(&self) -> bool {
        self.0.starts_with(INTERN_ID_PREFIX)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for MailboxId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<String> for MailboxId {
    fn from(id: String) -> Self {
        Self(id)
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
