use super::{GetState, QueryState};
use jmap_client::core::set::SetError;

pub struct QueryResponse<Id> {
    pub ids: Vec<Id>,
    pub state: QueryState,
}

pub struct GetOneResult<T> {
    pub value: T,
    pub state: GetState,
}

pub struct GetBatchResult<T, M> {
    pub values: T,
    pub not_found: M,
    pub state: GetState,
}

impl<T, M> GetBatchResult<T, M> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> GetBatchResult<U, M> {
        GetBatchResult {
            values: f(self.values),
            not_found: self.not_found,
            state: self.state,
        }
    }
}

pub struct CreateResult<T> {
    pub value: Result<T, SetError<String>>,
    pub state: GetState,
}

pub struct UpdateResult<Id, UpdatedData> {
    pub updated: Vec<UpdatedData>,
    pub failed: Vec<(Id, SetError<String>)>,
    pub new_state: GetState,
}

pub struct DestroyResult<Id> {
    pub destroyed: Vec<Id>,
    pub failed: Vec<(Id, SetError<String>)>,
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
