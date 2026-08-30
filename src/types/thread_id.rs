#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ThreadId(pub String);

impl From<&str> for ThreadId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<String> for ThreadId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<ThreadId> for String {
    fn from(id: ThreadId) -> Self {
        id.0
    }
}

impl From<&ThreadId> for String {
    fn from(id: &ThreadId) -> Self {
        id.0.clone()
    }
}
