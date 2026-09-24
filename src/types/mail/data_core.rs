use super::MailKeyword;
use crate::types::{MailId, MailboxId, ThreadId};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, Property};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailDataCore {
    pub id: MailId,
    pub message_id: Option<String>,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
    pub thread_id: ThreadId,
}

impl MailDataCore {
    pub const GET_REQUEST_PROPERTIES: [Property; 8] = [
        Property::Id,
        Property::MessageId,
        Property::Keywords,
        Property::Subject,
        Property::ReceivedAt,
        Property::HasAttachment,
        Property::MailboxIds,
        Property::ThreadId,
    ];

    pub fn from_get_request(mut mail: Email) -> Self {
        Self {
            id: mail.take_id().into(),
            message_id: mail
                .message_id()
                .map(|ids| ids.iter().next().cloned())
                .flatten(),
            keywords: mail.keywords().into_iter().map(MailKeyword::from).collect(),
            subject: mail.take_subject(),
            received_at: DateTime::<Utc>::from_timestamp(mail.received_at().unwrap(), 0)
                .expect("Valid timestamp")
                .with_timezone(&Local),
            has_attachment: mail.has_attachment(),
            mailbox_ids: mail
                .mailbox_ids()
                .into_iter()
                .map(|id| MailboxId(id.to_string()))
                .collect(),
            thread_id: mail.take_thread_id().unwrap().into(),
        }
    }
}
