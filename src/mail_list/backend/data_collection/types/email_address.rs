#[derive(Debug, Clone)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub address: String,
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr = self.address.as_str();
        if let Some(name) = &self.name {
            write!(f, "{name} <{addr}>")
        } else {
            write!(f, "{addr}")
        }
    }
}

impl From<jmap_client::email::EmailAddress> for EmailAddress {
    fn from(addr: jmap_client::email::EmailAddress) -> Self {
        Self::from(&addr)
    }
}

impl From<&jmap_client::email::EmailAddress> for EmailAddress {
    fn from(addr: &jmap_client::email::EmailAddress) -> Self {
        Self {
            name: addr.name().map(|name| name.to_string()).clone(),
            address: addr.email().to_string(),
        }
    }
}
