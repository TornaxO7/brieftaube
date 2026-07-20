const DRAFT: &str = "$draft";
const SEEN: &str = "$seen";
const FLAGGED: &str = "$flagged";
const ANSWERED: &str = "$answered";
const FORWARDED: &str = "$forwarded";
const PHISING: &str = "$phishing";
const JUNK: &str = "$junk";
const NOTJUNK: &str = "$notjunk";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MailKeyword {
    Draft,
    Seen,
    Flagged,
    Answered,
    Forwarded,
    Phising,
    Junk,
    Notjunk,
    Other(String),
}

impl std::fmt::Display for MailKeyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Draft => DRAFT,
            Self::Seen => SEEN,
            Self::Flagged => FLAGGED,
            Self::Answered => ANSWERED,
            Self::Forwarded => FORWARDED,
            Self::Phising => PHISING,
            Self::Junk => JUNK,
            Self::Notjunk => NOTJUNK,
            Self::Other(other) => other.as_str(),
        };

        write!(f, "{}", s)
    }
}

impl Into<String> for MailKeyword {
    fn into(self) -> String {
        self.to_string()
    }
}

impl From<&str> for MailKeyword {
    fn from(s: &str) -> Self {
        match s {
            DRAFT => Self::Draft,
            SEEN => Self::Seen,
            FLAGGED => Self::Flagged,
            ANSWERED => Self::Answered,
            FORWARDED => Self::Forwarded,
            PHISING => Self::Phising,
            JUNK => Self::Junk,
            NOTJUNK => Self::Notjunk,
            other => Self::Other(other.to_string()),
        }
    }
}
