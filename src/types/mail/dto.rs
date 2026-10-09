use crate::types::{MailAddresses, MailDataAttachment, MailId, MailKeyword, MailboxId, ThreadId};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, EmailBodyPart, Property};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MailDto {
    pub core: MailDtoCore,
    pub preview: Option<MailDtoPreview>,

    pub text_part_ids: Option<MailDtoTextPartIds>,
    pub html_part_ids: Option<MailDtoHtmlPartIds>,
    pub body_parts: HashMap<String, String>,
}

impl MailDto {
    pub fn new(stage0: MailDtoCore) -> Self {
        Self {
            core: stage0,
            preview: None,
            text_part_ids: None,
            html_part_ids: None,
            body_parts: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailDtoCore {
    pub id: MailId,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
    pub thread_id: ThreadId,
}

impl MailDtoCore {
    pub const GET_REQUEST_PROPERTIES: [Property; 6] = [
        Property::Keywords,
        Property::Subject,
        Property::ReceivedAt,
        Property::HasAttachment,
        Property::MailboxIds,
        Property::ThreadId,
    ];
}

impl From<jmap_client::email::Email> for MailDtoCore {
    fn from(mut jmap_mail: jmap_client::email::Email) -> Self {
        Self {
            id: jmap_mail.take_id().into(),
            keywords: jmap_mail
                .keywords()
                .into_iter()
                .map(MailKeyword::from)
                .collect(),
            subject: jmap_mail.take_subject(),
            received_at: DateTime::<Utc>::from_timestamp(jmap_mail.received_at().unwrap(), 0)
                .map(|time| time.with_timezone(&Local))
                .unwrap(),
            has_attachment: jmap_mail.has_attachment(),
            mailbox_ids: jmap_mail
                .mailbox_ids()
                .into_iter()
                .map(|id| MailboxId(id.to_string()))
                .collect(),
            thread_id: jmap_mail.take_thread_id().map(ThreadId::from).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailDtoPreview {
    pub from: Option<MailAddresses>,
    pub to: Option<MailAddresses>,
    pub cc: Option<MailAddresses>,
    pub bcc: Option<MailAddresses>,
    pub preview: Option<String>,
    pub attachments: Option<Vec<MailDataAttachment>>,
}

impl MailDtoPreview {
    pub const GET_REQUEST_PROPERTIES: [Property; 6] = [
        Property::From,
        Property::To,
        Property::Cc,
        Property::Bcc,
        Property::Preview,
        Property::Attachments,
    ];
}

impl From<jmap_client::email::Email> for MailDtoPreview {
    fn from(mut jmap_mail: jmap_client::email::Email) -> Self {
        Self {
            from: jmap_mail.take_from().map(MailAddresses::from),
            to: jmap_mail.take_to().map(MailAddresses::from),
            cc: jmap_mail.take_cc().map(MailAddresses::from),
            bcc: jmap_mail.take_bcc().map(MailAddresses::from),
            preview: jmap_mail.take_preview(),
            attachments: jmap_mail
                .attachments()
                .map(|parts| parts.iter().map(MailDataAttachment::from).collect()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailDtoTextPartIds(pub Vec<String>);

impl MailDtoTextPartIds {
    pub const GET_REQUEST_PROPERTIES: [Property; 2] = [Property::TextBody, Property::BodyValues];

    pub fn new(jmap_mail: Email) -> (Self, HashMap<String, String>) {
        let (part_ids, body_values) =
            collect_part_ids_and_body_values(&jmap_mail, jmap_mail.text_body().unwrap());

        (Self(part_ids), body_values)
    }
}

#[derive(Debug, Clone)]
pub struct MailDtoHtmlPartIds(pub Vec<String>);

impl MailDtoHtmlPartIds {
    pub const GET_REQUEST_PROPERTIES: [Property; 2] = [Property::HtmlBody, Property::BodyValues];

    pub fn new(jmap_mail: Email) -> (Self, HashMap<String, String>) {
        let (part_ids, body_values) =
            collect_part_ids_and_body_values(&jmap_mail, jmap_mail.html_body().unwrap());

        (Self(part_ids), body_values)
    }
}

fn collect_part_ids_and_body_values(
    jmap_mail: &Email,
    body_parts: &[EmailBodyPart],
) -> (Vec<String>, HashMap<String, String>) {
    let mut body_parts_mapping = HashMap::new();
    let mut part_ids = Vec::new();

    for body_part in body_parts {
        let part_id = body_part.part_id().expect("Part id exists");

        part_ids.push(part_id.to_string());
        body_parts_mapping.insert(
            part_id.to_string(),
            jmap_mail
                .body_value(part_id)
                .expect("Body value exists")
                .value()
                .to_string(),
        );
    }

    (part_ids, body_parts_mapping)
}
