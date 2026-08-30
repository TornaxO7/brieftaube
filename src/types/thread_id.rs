#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ThreadId(pub String);

impl From<&str> for ThreadId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}
