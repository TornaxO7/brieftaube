use crate::{
    types::{AccountData, AccountId},
    ui::mailfs::types::Selectable,
};

pub struct AccountEntry {
    pub id: AccountId,
    pub name: String,
}

impl From<AccountData> for AccountEntry {
    fn from(data: AccountData) -> Self {
        Self {
            id: data.id,
            name: data.name,
        }
    }
}

impl Selectable for AccountEntry {
    fn is_selected(&self) -> bool {
        false
    }
}
