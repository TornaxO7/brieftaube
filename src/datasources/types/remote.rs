use jmap_client::core::set::SetError;

use crate::types::{MailId, MailUpdate};

use super::{GetState, QueryState};

pub struct QueryResponse<Id> {
    pub ids: Vec<Id>,
    pub state: QueryState,
}

pub struct GetResult<Id, T> {
    pub values: Vec<(Id, T)>,
    pub not_found: Vec<Id>,
    pub state: GetState,
}

pub struct SetResult<T> {
    pub value: T,
    pub state: GetState,
}

pub struct CreateResult<T> {
    pub value: Result<T, SetError<String>>,
    pub state: GetState,
}

pub struct UpdateResult {
    pub updated: Vec<(MailId, MailUpdate)>,
    pub failed: Vec<(MailId, SetError<String>)>,
    pub new_state: GetState,
}

pub struct GetChangeResult<Id> {
    pub new_state: GetState,
    pub has_more_changes: bool,
    pub created: Vec<Id>,
    pub updated: Vec<Id>,
    pub destroyed: Vec<Id>,
}

pub struct QueryChangeResult<Id> {
    pub new_state: QueryState,
    pub removed: Vec<Id>,
    pub added: Vec<(Id, usize)>,
}
