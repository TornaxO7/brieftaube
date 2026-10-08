use super::MailKeyword;
use crate::types::{MailDto, MailboxId, ThreadId};
use chrono::{DateTime, Local};
use jmap_client::email::Property;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailDataCore {
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: Option<DateTime<Local>>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
    pub thread_id: Option<ThreadId>,
}

impl MailDataCore {
    pub const GET_REQUEST_PROPERTIES: [Property; 6] = [
        Property::Keywords,
        Property::Subject,
        Property::ReceivedAt,
        Property::HasAttachment,
        Property::MailboxIds,
        Property::ThreadId,
    ];

    // pub fn from_get_request(mut mail: Email) -> Self {
    //     Self {
    //         keywords: mail.keywords().into_iter().map(MailKeyword::from).collect(),
    //         subject: mail.take_subject(),
    //         received_at: DateTime::<Utc>::from_timestamp(mail.received_at().unwrap(), 0)
    //             .map(|timezone| timezone.with_timezone(&Local)),
    //         has_attachment: mail.has_attachment(),
    //         mailbox_ids: mail
    //             .mailbox_ids()
    //             .into_iter()
    //             .map(|id| MailboxId(id.to_string()))
    //             .collect(),
    //         thread_id: mail.take_thread_id().unwrap().into(),
    //     }
    // }
}

impl From<MailDto> for MailDataCore {
    fn from(cached_mail: MailDto) -> Self {
        let stage0 = cached_mail.core;

        Self {
            keywords: stage0.keywords,
            subject: stage0.subject,
            received_at: stage0.received_at.map(|time| time.with_timezone(&Local)),
            has_attachment: stage0.has_attachment,
            mailbox_ids: stage0.mailbox_ids,
            thread_id: stage0.thread_id,
        }
    }
}
