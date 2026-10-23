use crate::config::{UserConfig, Username};

#[derive(Debug, Clone)]
pub struct UserEntry {
    pub username: Username,
}

impl From<UserConfig> for UserEntry {
    fn from(conf: UserConfig) -> Self {
        Self {
            username: conf.username,
        }
    }
}
