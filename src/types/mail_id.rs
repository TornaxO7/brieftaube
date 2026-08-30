#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailId(pub String);

impl MailId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<MailId> for String {
    fn from(id: MailId) -> Self {
        id.0.clone()
    }
}

impl From<&MailId> for String {
    fn from(id: &MailId) -> Self {
        id.0.clone()
    }
}

impl From<String> for MailId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&String> for MailId {
    fn from(id: &String) -> Self {
        Self(id.clone())
    }
}

impl From<&str> for MailId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}
