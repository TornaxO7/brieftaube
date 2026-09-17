use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Username(pub String);

impl From<String> for Username {
    fn from(username: String) -> Self {
        Self(username)
    }
}

impl From<&str> for Username {
    fn from(username: &str) -> Self {
        Self(username.to_string())
    }
}

impl Username {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
