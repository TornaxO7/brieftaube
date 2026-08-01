use super::{MailAddress, MailKeyword};
use crate::backend::{mailbox::types::MailboxId, mails::types::MailId, threads::types::ThreadId};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, EmailBodyPart, Property};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailData {
    pub id: MailId,
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

    pub text_body: Option<MailDataTextBody>,
    pub html_body: Option<MailDataHtmlBody>,
    pub attachments: Option<Vec<MailDataAttachment>>,
}

impl MailData {
    pub const PROPERTIES: [Property; 11] = [
        Property::Id,
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

    pub fn new(mut mail: Email) -> Self {
        Self {
            id: MailId(mail.take_id()),
            thread_id: ThreadId(mail.take_thread_id().unwrap()),
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
                .map(|id| MailboxId(id.to_owned()))
                .collect(),

            text_body: None,
            html_body: None,
            attachments: (!mail.has_attachment()).then_some(vec![]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataTextBody(pub String);

impl MailDataTextBody {
    pub fn new(mail: &Email) -> Self {
        let parts = mail.text_body().unwrap();
        let content = join_body_values(mail, parts).unwrap();

        Self(content)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataHtmlBody(pub String);

impl MailDataHtmlBody {
    pub fn new(mail: &Email) -> Self {
        let parts = mail.html_body().unwrap();
        let content = join_body_values(mail, parts).unwrap();

        Self(content)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailDataAttachment {
    pub name: String,
    pub content_type: String,
    pub size: usize,
    pub blob_id: Option<String>,
}

impl From<&EmailBodyPart> for MailDataAttachment {
    fn from(part: &EmailBodyPart) -> Self {
        Self {
            name: part.name().unwrap().to_string(),
            content_type: part.content_type().unwrap().to_string(),
            size: part.size(),
            blob_id: part.blob_id().map(ToString::to_string),
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
