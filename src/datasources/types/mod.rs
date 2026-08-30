pub mod cache;
pub mod remote;

use std::ops::Range;

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
