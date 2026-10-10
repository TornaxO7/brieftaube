use super::MailAddress;
use jmap_client::email::EmailAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAddresses(pub Vec<MailAddress>);

impl std::fmt::Display for MailAddresses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .0
            .split_first()
            .map(|(first, rest)| {
                rest.iter().fold(format!("{}", first), |acc, addr| {
                    format!("{acc}, {}", addr.to_string())
                })
            })
            .unwrap_or(String::new());

        write!(f, "{}", s)
    }
}

impl IntoIterator for MailAddresses {
    type Item = MailAddress;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<EmailAddress>> for MailAddresses {
    fn from(addresses: Vec<EmailAddress>) -> Self {
        Self::from(addresses.as_slice())
    }
}

impl From<&[EmailAddress]> for MailAddresses {
    fn from(addresses: &[EmailAddress]) -> Self {
        Self(addresses.into_iter().map(MailAddress::from).collect())
    }
}

impl AsRef<[MailAddress]> for MailAddresses {
    fn as_ref(&self) -> &[MailAddress] {
        &self.0
    }
}
