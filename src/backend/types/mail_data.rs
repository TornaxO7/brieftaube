use super::{MailAddress, MailKeyword};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, EmailBodyPart, Property};
use std::collections::HashSet;
use tokio::sync::watch;

#[derive(Debug, Clone, Default)]
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

    pub text_body: Loadable<MailDataTextBody>,
    pub html_body: Loadable<MailDataHtmlBody>,
    pub attachments: Loadable<Vec<MailDataAttachment>>,
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

    pub fn get_body(&self, ty: MailBodyType) -> Loadable<&str> {
        match ty {
            MailBodyType::Text => self.text_body.as_ref().map(|body| body.0.as_str()),
            MailBodyType::Html => self.html_body.as_ref().map(|body| body.0.as_str()),
        }
    }

    pub fn init_body(&mut self, ty: MailBodyType, rx: watch::Receiver<()>) {
        match ty {
            MailBodyType::Text => self.text_body = Loadable::requesting(rx),
            MailBodyType::Html => self.html_body = Loadable::requesting(rx),
        }
    }

    pub fn init_attachments(&mut self, notifier: watch::Receiver<()>) {
        self.attachments = Loadable::Requested { notifier: notifier };
    }
}

impl From<jmap_client::email::Email> for MailData {
    fn from(mut mail: jmap_client::email::Email) -> Self {
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

            text_body: match MailDataTextBody::new(&mail) {
                Some(text) => Loadable::Loaded(text),
                None => Loadable::NotRequested,
            },
            html_body: match MailDataHtmlBody::new(&mail) {
                Some(html) => Loadable::Loaded(html),
                None => Loadable::NotRequested,
            },
            attachments: if !mail.has_attachment() {
                Loadable::Loaded(vec![])
            } else {
                Loadable::NotRequested
            },
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
