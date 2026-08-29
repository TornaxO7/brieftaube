use std::ops::Range;

pub type GetState = String;
pub type QueryState = String;

pub struct QueryWindow {
    pub start: usize,
    pub limit: usize,
}

pub struct QueryResponse<Id> {
    pub values: Vec<Vec<Id>>,
    pub missing: Vec<Range<usize>>,
    pub query_state: Option<QueryState>,
}

pub enum Coverage {
    Complete,
    // contains the missing start position and length
    Partial { start: usize, len: usize },
}

pub struct GetResult<T> {
    pub value: T,
    pub state: GetState,
}

impl<T> GetResult<T> {
    pub fn as_ref(&self) -> GetResult<&T> {
        GetResult {
            value: &self.value,
            state: self.state.clone(),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> GetResult<U> {
        GetResult {
            value: f(self.value),
            state: self.state,
        }
    }
}

pub struct SetResult<T> {
    pub value: T,
    pub state: GetState,
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
    pub added: Vec<Id>,
}
