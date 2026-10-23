use crate::types::{AccountData, AccountId};

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
