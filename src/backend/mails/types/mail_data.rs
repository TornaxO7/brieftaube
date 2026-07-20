use super::{MailAddress, MailKeyword, ThreadId};
use crate::backend::mailbox::types::MailboxId;
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, EmailBodyPart, Property};
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

    pub rest: Option<MailDataRest>,
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
            rest: None,
        }
    }

    pub fn extend(&mut self, mail: &Email) {
        self.rest = Some(MailDataRest::new(mail));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailDataRest {
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<MailDataAttachment>,
}

impl MailDataRest {
    pub const PROPERTIES: [Property; 5] = [
        Property::Id,
        Property::TextBody,
        Property::HtmlBody,
        Property::BodyValues,
        Property::Attachments,
    ];

    pub fn new(mail: &Email) -> Self {
        let text_body = mail
            .text_body()
            .and_then(|text_body| join_body_values(mail, text_body));

        let html_body = mail
            .html_body()
            .and_then(|html_body| join_body_values(mail, html_body));

        let attachments = mail
            .attachments()
            .map(|parts| parts.iter().map(MailDataAttachment::from).collect())
            .unwrap_or_default();

        Self {
            text_body,
            html_body,
            attachments,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailDataAttachment {
    pub name: Option<String>,
    pub content_type: Option<String>,
    pub size: usize,
    pub blob_id: Option<String>,
}

impl From<&EmailBodyPart> for MailDataAttachment {
    fn from(part: &EmailBodyPart) -> Self {
        Self {
            name: part.name().map(ToString::to_string),
            content_type: part.content_type().map(ToString::to_string),
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
