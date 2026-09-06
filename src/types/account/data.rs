use crate::types::AccountId;

#[derive(Debug, Clone)]
pub struct AccountData {
    pub id: AccountId,
    pub name: String,
}
