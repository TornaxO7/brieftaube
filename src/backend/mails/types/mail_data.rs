use super::{MailAddress, MailKeyword, ThreadId};
use crate::backend::mailbox::types::MailboxId;
use chrono::{DateTime, Local, Utc};
use jmap_client::email::Property;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailData {
    pub id: String,
    pub thread_id: ThreadId,
    pub keywords: HashSet<MailKeyword>,
    pub from: Vec<MailAddress>,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub subject: String,
    pub preview: String,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: HashSet<MailboxId>,
}

impl MailData {
    pub const PROPERTIES: [Property; 11] = [
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
        jmap_client::email::Property::MailboxIds,
    ];
}

impl From<jmap_client::email::Email> for MailData {
    fn from(mut mail: jmap_client::email::Email) -> Self {
        Self {
            id: mail.take_id(),
            thread_id: mail.take_thread_id().unwrap(),
            keywords: mail.keywords().into_iter().map(MailKeyword::from).collect(),
            from: mail
                .take_from()
                .map(|addresses| addresses.into_iter().map(MailAddress::from).collect())
                .unwrap_or(vec![]),
            to: mail
                .to()
                .map(|addresses| addresses.into_iter().map(MailAddress::from).collect())
                .unwrap_or(vec![]),
            cc: mail
                .take_cc()
                .map(|cc| cc.into_iter().map(MailAddress::from).collect())
                .unwrap_or(vec![]),
            subject: mail.take_subject().unwrap(),
            preview: mail.take_preview().unwrap(),
            received_at: DateTime::<Utc>::from_timestamp(mail.received_at().unwrap(), 0)
                .expect("Valid timestamp")
                .with_timezone(&Local),
            has_attachment: mail.has_attachment(),
            mailbox_ids: mail
                .mailbox_ids()
                .into_iter()
                .map(|id| id.to_owned())
                .collect(),
        }
    }
}
