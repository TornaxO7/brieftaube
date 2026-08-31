pub mod cache;
pub mod remote;

use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetState(pub String);

impl From<String> for GetState {
    fn from(state: String) -> Self {
        Self(state)
    }
}

impl AsRef<str> for GetState {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<GetState> for String {
    fn from(state: GetState) -> Self {
        state.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryState(pub String);

impl From<String> for QueryState {
    fn from(state: String) -> Self {
        Self(state)
    }
}

impl From<&str> for QueryState {
    fn from(state: &str) -> Self {
        Self(state.to_string())
    }
}

impl AsRef<str> for QueryState {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<QueryState> for String {
    fn from(state: QueryState) -> Self {
        state.0
    }
}

#[derive(Debug, Clone)]
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
