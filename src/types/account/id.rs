#[derive(Debug, Clone)]
pub struct AccountId(pub String);

impl From<String> for AccountId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
