use crate::{
    config::{UserConfig, Username},
    ui::mailfs::types::Selectable,
};

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

impl Selectable for UserEntry {
    fn is_selected(&self) -> bool {
        false
    }

    fn set_selection(&mut self, _selected: bool) {}
}
