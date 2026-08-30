use super::{MailAddress, MailId, MailKeyword, MailboxId, ThreadId};
use crate::types::MailNew;
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, EmailBodyPart, Property};
use std::collections::HashSet;

// TODO: Create `MailAddresses`
#[derive(Debug, Clone, Default)]
pub struct MailData {
    pub id: MailId,
    pub message_id: Option<String>,
    pub thread_id: ThreadId,
    pub keywords: HashSet<MailKeyword>,
    pub from: Option<Vec<MailAddress>>,
    pub to: Option<Vec<MailAddress>>,
    pub cc: Option<Vec<MailAddress>>,
    pub bcc: Option<Vec<MailAddress>>,
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
            from: mail
                .take_from()
                .map(|addresses| addresses.into_iter().map(MailAddress::from).collect()),
            to: mail
                .to()
                .map(|addresses| addresses.into_iter().map(MailAddress::from).collect()),
            cc: mail
                .take_cc()
                .map(|cc| cc.into_iter().map(MailAddress::from).collect()),
            bcc: mail
                .take_bcc()
                .map(|bcc| bcc.into_iter().map(MailAddress::from).collect()),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataTextBody(pub String);

impl MailDataTextBody {
    pub fn new(mail: &Email) -> Option<Self> {
        let parts = mail.text_body()?;
        let content = join_body_values(mail, parts)?;

        Some(Self(content))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataHtmlBody(pub String);

impl MailDataHtmlBody {
    pub fn new(mail: &Email) -> Option<Self> {
        let parts = mail.html_body()?;
        let content = join_body_values(mail, parts)?;

        Some(Self(content))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailDataAttachment {
    pub name: String,
    pub content_type: String,
    pub size: usize,
    pub blob_id: String,
}

impl From<&EmailBodyPart> for MailDataAttachment {
    fn from(part: &EmailBodyPart) -> Self {
        Self {
            name: part.name().unwrap().to_owned(),
            content_type: part.content_type().unwrap().to_owned(),
            size: part.size(),
            blob_id: part.blob_id().unwrap().to_owned(),
        }
    }
}

fn join_body_values(mail: &Email, parts: &[EmailBodyPart]) -> Option<String> {
    let mut body = String::new();

    for part in parts {
        let Some(part_id) = part.part_id() else {
            continue;
        };

        if let Some(value) = mail.body_value(part_id) {
            body.push_str(value.value());
        }
    }

    if body.is_empty() { None } else { Some(body) }
}
