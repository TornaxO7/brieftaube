use super::EmailAddress;
use crate::utils::{EmailKeyword, ThreadId};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::Property;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailData {
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

impl MailData {
    pub const PROPERTIES: [Property; 10] = [
        jmap_client::email::Property::Id,
        jmap_client::email::Property::ThreadId,
        jmap_client::email::Property::Keywords,
        jmap_client::email::Property::From,
        jmap_client::email::Property::To,
        jmap_client::email::Property::Cc,
        jmap_client::email::Property::Subject,
        jmap_client::email::Property::Preview,
        jmap_client::email::Property::ReceivedAt,
        jmap_client::email::Property::HasAttachment,
    ];
}

impl From<jmap_client::email::Email> for MailData {
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
