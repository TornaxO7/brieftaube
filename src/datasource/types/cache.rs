use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct QueryResponseSection<T> {
    pub start: usize,
    pub values: Vec<T>, // consecutive in this section
}

impl<T> QueryResponseSection<T> {
    fn map<U, F>(self, f: &F) -> QueryResponseSection<U>
    where
        F: Fn(T) -> U,
    {
        QueryResponseSection {
            start: self.start,
            values: self.values.into_iter().map(|value| f(value)).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryResponse<T> {
    pub values: Vec<QueryResponseSection<T>>,
    pub missing: Vec<Range<usize>>,
}

impl<T> QueryResponse<T> {
    pub fn map<U>(self, f: impl Fn(T) -> U) -> QueryResponse<U> {
        let values = self
            .values
            .into_iter()
            .map(|section| section.map(&f))
            .collect();

        QueryResponse {
            values,
            missing: self.missing,
        }
    }
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
