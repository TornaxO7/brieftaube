use super::{MailId, MailKeyword, MailboxId, ThreadId};
use crate::types::{MailAddresses, MailNew, MailUpdate};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, EmailBodyPart, Property};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct MailData {
    pub id: MailId,
    pub message_id: Option<String>,
    pub thread_id: ThreadId,
    pub keywords: HashSet<MailKeyword>,
    pub from: Option<MailAddresses>,
    pub to: Option<MailAddresses>,
    pub cc: Option<MailAddresses>,
    pub bcc: Option<MailAddresses>,
    pub subject: Option<String>,
    pub preview: String,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: HashSet<MailboxId>,
}

impl MailData {
    pub const PROPERTIES: [Property; 12] = [
        Property::Id,
        Property::MessageId,
        Property::ThreadId,
        Property::Keywords,
        Property::From,
        Property::To,
        Property::Cc,
        Property::Subject,
        Property::Preview,
        Property::ReceivedAt,
        Property::HasAttachment,
        Property::MailboxIds,
    ];

    pub fn from_new(new: MailNew, mut server_mail: Email) -> Self {
        Self {
            id: server_mail.take_id().into(),
            message_id: server_mail
                .message_id()
                .map(|ids| ids.iter().next().cloned())
                .flatten(),
            thread_id: server_mail
                .thread_id()
                .expect("Server returns thread id")
                .into(),
            keywords: new.keywords,
            from: new.from,
            to: new.to,
            cc: new.cc,
            bcc: new.bcc,
            subject: new.subject,
            preview: server_mail.take_preview().unwrap_or_default(),
            received_at: DateTime::<Utc>::from_timestamp(server_mail.received_at().unwrap(), 0)
                .unwrap()
                .with_timezone(&Local),
            mailbox_ids: new.mailbox_ids,
            has_attachment: false,
        }
    }

    pub fn from_get_request(mut mail: Email) -> Self {
        Self {
            id: MailId(mail.take_id()),
            message_id: mail
                .message_id()
                .map(|ids| ids.iter().next().cloned())
                .flatten(),
            thread_id: ThreadId(mail.take_thread_id().unwrap()),
            keywords: mail.keywords().into_iter().map(MailKeyword::from).collect(),
            from: mail.take_from().map(MailAddresses::from),
            to: mail.to().map(MailAddresses::from),
            cc: mail.take_cc().map(MailAddresses::from),
            bcc: mail.take_bcc().map(MailAddresses::from),
            subject: mail.take_subject(),
            preview: mail.take_preview().unwrap_or_default(),
            received_at: DateTime::<Utc>::from_timestamp(mail.received_at().unwrap(), 0)
                .expect("Valid timestamp")
                .with_timezone(&Local),
            has_attachment: mail.has_attachment(),
            mailbox_ids: mail
                .mailbox_ids()
                .into_iter()
                .map(|id| MailboxId(id.to_owned()))
                .collect(),
        }
    }

    pub fn update(&mut self, update: MailUpdate) {
        if let Some(new_keywords) = update.patch_keywords {
            for (keyword, set) in new_keywords {
                if set {
                    self.keywords.insert(keyword);
                } else {
                    self.keywords.remove(&keyword);
                }
            }
        }

        if let Some(mailbox_ids) = update.mailbox_ids {
            for (mailbox_id, set) in mailbox_ids {
                if set {
                    self.mailbox_ids.insert(mailbox_id);
                } else {
                    self.mailbox_ids.remove(&mailbox_id);
                }
            }
        }
    }
}
