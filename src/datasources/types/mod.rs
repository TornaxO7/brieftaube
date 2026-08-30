use std::ops::Range;

use jmap_client::{core::changes::ChangesResponse, email::Email};

pub type GetState = String;
pub type QueryState = String;

pub struct QueryWindow {
    pub start: u32,
    pub limit: usize,
}

impl QueryWindow {
    pub fn as_range(&self) -> Range<usize> {
        let start = self.start as usize;
        let end = start + self.limit;
        start..end
    }
}

impl From<Range<usize>> for QueryWindow {
    fn from(range: Range<usize>) -> Self {
        let start = range.start as u32;
        let limit = range.len();

        Self { start, limit }
    }
}

pub struct RemoteQueryResponse<Id> {
    pub ids: Vec<Id>,
    pub state: QueryState,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalQueryResponseSection<Id> {
    pub start: usize,
    pub ids: Vec<Id>, // consecutive ids
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalQueryResponse<Id> {
    pub values: Vec<LocalQueryResponseSection<Id>>,
    pub missing: Vec<Range<usize>>,
    pub query_state: Option<QueryState>,
}

pub struct RemoteGetResult<Id, T> {
    pub values: T,
    pub not_found: Vec<Id>,
    pub state: GetState,
}

pub struct RemoteSetResult<T> {
    pub value: T,
    pub state: GetState,
}

pub struct LocalGetResult<T> {
    pub value: T,
    pub state: Option<GetState>,
}

impl<T> LocalGetResult<T> {
    pub fn as_ref(&self) -> LocalGetResult<&T> {
        LocalGetResult {
            value: &self.value,
            state: self.state.clone(),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> LocalGetResult<U> {
        LocalGetResult {
            value: f(self.value),
            state: self.state,
        }
    }
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
