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

pub struct GetResult<T> {
    pub value: T,
    pub state: Option<GetState>,
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
