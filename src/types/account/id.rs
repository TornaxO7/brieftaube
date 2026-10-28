#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

impl AccountId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for AccountId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
