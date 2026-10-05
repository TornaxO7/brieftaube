#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BlobId(pub String);

impl BlobId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for BlobId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
