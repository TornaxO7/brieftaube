const DRAFT: &str = "$draft";
const SEEN: &str = "$seen";
const FLAGGED: &str = "$flagged";
const ANSWERED: &str = "$answered";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EmailKeyword {
    Draft,
    Seen,
    Flagged,
    Answered,
    Other(String),
}

impl EmailKeyword {
    pub fn as_str(&self) -> String {
        match self {
            Self::Draft => DRAFT.to_string(),
            Self::Seen => SEEN.to_string(),
            Self::Flagged => FLAGGED.to_string(),
            Self::Answered => ANSWERED.to_string(),
            Self::Other(other) => other.clone(),
        }
    }
}

impl From<&str> for EmailKeyword {
    fn from(s: &str) -> Self {
        match s {
            DRAFT => Self::Draft,
            SEEN => Self::Seen,
            FLAGGED => Self::Flagged,
            ANSWERED => Self::Answered,
            other => Self::Other(other.to_string()),
        }
    }
}
