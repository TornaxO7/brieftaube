use std::collections::HashSet;

use crate::utils::{EmailKeyword, ThreadId};
use chrono::{DateTime, Local, Utc};

#[derive(Debug, Clone)]
pub struct RootMailData {
    pub id: String,
    pub thread_id: ThreadId,
    pub keywords: HashSet<EmailKeyword>,
    pub from: Vec<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub subject: String,
    pub preview: String,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
}

impl From<jmap_client::email::Email> for RootMailData {
    fn from(mut mail: jmap_client::email::Email) -> Self {
        Self {
            id: mail.take_id(),
            thread_id: mail.take_thread_id().unwrap(),
            keywords: mail
                .keywords()
                .into_iter()
                .map(EmailKeyword::from)
                .collect(),
            from: mail
                .take_from()
                .map(|addresses| addresses.into_iter().map(EmailAddress::from).collect())
                .unwrap_or(vec![]),
            to: mail
                .to()
                .map(|addresses| addresses.into_iter().map(EmailAddress::from).collect())
                .unwrap_or(vec![]),
            cc: mail
                .take_cc()
                .map(|cc| cc.into_iter().map(EmailAddress::from).collect())
                .unwrap_or(vec![]),
            subject: mail.take_subject().unwrap(),
            preview: mail.take_preview().unwrap(),
            received_at: DateTime::<Utc>::from_timestamp(mail.received_at().unwrap(), 0)
                .expect("Valid timestamp")
                .with_timezone(&Local),
            has_attachment: mail.has_attachment(),
        }
    }
}

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
