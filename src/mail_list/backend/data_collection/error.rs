use crate::utils::ThreadId;

#[derive(Debug, Clone)]
pub enum UnfoldRowError {
    NonRootRow,
    NotInitialised(ThreadId),
    AlreadyUnfolded,
}
