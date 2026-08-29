#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailId(pub String);

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
