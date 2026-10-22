use crate::{
    types::{AccountData, AccountId},
    ui::mailfs::types::Selectable,
};

pub struct AccountEntry {
    pub id: AccountId,
    pub name: String,

    pub is_selected: bool,
}

impl From<AccountData> for AccountEntry {
    fn from(data: AccountData) -> Self {
        Self {
            id: data.id,
            name: data.name,

            is_selected: false,
        }
    }
}

impl Selectable for AccountEntry {
    fn is_selected(&self) -> bool {
        self.is_selected
    }
}
