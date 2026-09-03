use super::MailKeyword;
use crate::types::MailboxId;
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, Property};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailDataCore {
    pub message_id: Option<String>,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
}

impl MailDataCore {
    pub const GET_REQUEST_PROPERTIES: [Property; 7] = [
        Property::Id,
        Property::MessageId,
        Property::Keywords,
        Property::Subject,
        Property::ReceivedAt,
        Property::HasAttachment,
        Property::MailboxIds,
    ];

    pub fn from_get_request(mut mail: Email) -> Self {
        Self {
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
        }
    }
}
