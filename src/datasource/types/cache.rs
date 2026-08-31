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
}

pub struct GetBatchResult<T, M> {
    pub value: T,
    pub missing: M,
}

impl<T, M> GetBatchResult<T, M> {
    pub fn as_ref(&self) -> GetBatchResult<&T, &M> {
        GetBatchResult {
            value: &self.value,
            missing: &self.missing,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> GetBatchResult<U, M> {
        GetBatchResult {
            value: f(self.value),
            missing: self.missing,
        }
    }
}
