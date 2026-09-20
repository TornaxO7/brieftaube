#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

impl From<String> for AccountId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
