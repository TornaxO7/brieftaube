#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct IdentityId(pub String);

impl IdentityId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<IdentityId> for String {
    fn from(id: IdentityId) -> Self {
        id.0.clone()
    }
}

impl From<&IdentityId> for String {
    fn from(id: &IdentityId) -> Self {
        id.0.clone()
    }
}

impl From<String> for IdentityId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&String> for IdentityId {
    fn from(id: &String) -> Self {
        Self(id.clone())
    }
}

impl From<&str> for IdentityId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}
