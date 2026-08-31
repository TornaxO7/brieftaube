use super::{GetState, QueryState};
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct QueryResponseSection<Id> {
    pub start: usize,
    pub ids: Vec<Id>, // consecutive ids
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryResponse<Id> {
    pub values: Vec<QueryResponseSection<Id>>,
    pub missing: Vec<Range<usize>>,
    pub query_state: Option<QueryState>,
}

impl<Id> QueryResponse<Id> {
    pub fn is_initialised(&self) -> bool {
        self.query_state.is_none()
    }
}

pub struct GetOneResult<T> {
    pub value: T,
    pub state: Option<GetState>,
}

pub struct GetBatchResult<T, M> {
    pub value: T,
    pub missing: M,
    pub state: Option<GetState>,
}

impl<T, M> GetBatchResult<T, M> {
    pub fn as_ref(&self) -> GetBatchResult<&T, &M> {
        GetBatchResult {
            value: &self.value,
            missing: &self.missing,
            state: self.state.clone(),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> GetBatchResult<U, M> {
        GetBatchResult {
            value: f(self.value),
            missing: self.missing,
            state: self.state,
        }
    }
}
