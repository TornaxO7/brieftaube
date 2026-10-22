use crate::{
    config::{UserConfig, Username},
    ui::mailfs::types::Selectable,
};

#[derive(Debug, Clone)]
pub struct UserEntry {
    pub username: Username,
    pub is_selected: bool,
}

impl From<UserConfig> for UserEntry {
    fn from(conf: UserConfig) -> Self {
        Self {
            username: conf.username,
            is_selected: false,
        }
    }
}

impl Selectable for UserEntry {
    fn is_selected(&self) -> bool {
        self.is_selected
    }
}
