use crate::types::{
    MailAddresses, MailDataAttachment, MailId, MailKeyword, MailProperty, MailboxId, ThreadId,
};
use chrono::{DateTime, Local, Utc};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

#[derive(Debug, Clone)]
pub struct CachedMail {
    pub id: MailId,
    pub keywords: OnceLock<HashSet<MailKeyword>>,
    pub mailbox_ids: OnceLock<Vec<MailboxId>>,
    pub subject: OnceLock<String>,
    pub received_at: OnceLock<DateTime<Local>>,
    pub has_attachment: OnceLock<bool>,
    pub thread_id: OnceLock<ThreadId>,

    pub from: OnceLock<MailAddresses>,
    pub to: OnceLock<MailAddresses>,
    pub cc: OnceLock<MailAddresses>,
    pub bcc: OnceLock<MailAddresses>,
    pub preview: OnceLock<String>,
    pub attachments: OnceLock<Vec<MailDataAttachment>>,

    pub text_part_ids: OnceLock<Vec<String>>,
    pub html_part_ids: OnceLock<Vec<String>>,
    pub body_parts: OnceLock<HashMap<String, String>>,
}

impl CachedMail {
    pub fn has_properties(&self, properties: &[MailProperty]) -> bool {
        properties.iter().all(|property| match property {
            MailProperty::Id => true,
            MailProperty::Keywords => self.keywords.get().is_some(),
            MailProperty::MailboxIds => self.mailbox_ids.get().is_some(),
            MailProperty::Subject => self.subject.get().is_some(),
            MailProperty::ReceivedAt => self.received_at.get().is_some(),
            MailProperty::HasAttachment => self.has_attachment.get().is_some(),
            MailProperty::ThreadId => self.thread_id.get().is_some(),
            MailProperty::From => self.from.get().is_some(),
            MailProperty::To => self.to.get().is_some(),
            MailProperty::Cc => self.cc.get().is_some(),
            MailProperty::Bcc => self.bcc.get().is_some(),
            MailProperty::Preview => self.preview.get().is_some(),
            MailProperty::Attachments => self.attachments.get().is_some(),
            MailProperty::TextBody => self.text_part_ids.get().is_some(),
            MailProperty::HtmlBody => self.html_part_ids.get().is_some(),
        })
    }

    pub fn merge(&mut self, mut other: CachedMail) {
        self.id = other.id;

        if let Some(keywords) = other.keywords.take() {
            self.keywords = OnceLock::from(keywords);
        }

        if let Some(mailbox_ids) = other.mailbox_ids.take() {
            self.mailbox_ids = OnceLock::from(mailbox_ids);
        }

        if let Some(subject) = other.subject.take() {
            self.subject = OnceLock::from(subject);
        }

        if let Some(received_at) = other.received_at.take() {
            self.received_at = OnceLock::from(received_at);
        }

        if let Some(has_attachment) = other.has_attachment.take() {
            self.has_attachment = OnceLock::from(has_attachment);
        }

        if let Some(thread_id) = other.thread_id.take() {
            self.thread_id = OnceLock::from(thread_id);
        }

        if let Some(from) = other.from.take() {
            self.from = OnceLock::from(from);
        }

        if let Some(to) = other.to.take() {
            self.to = OnceLock::from(to);
        }

        if let Some(cc) = other.cc.take() {
            self.cc = OnceLock::from(cc);
        }

        if let Some(bcc) = other.bcc.take() {
            self.bcc = OnceLock::from(bcc);
        }

        if let Some(preview) = other.preview.take() {
            self.preview = OnceLock::from(preview);
        }

        if let Some(attachments) = other.attachments.take() {
            self.attachments = OnceLock::from(attachments);
        }

        if let Some(text_part_ids) = other.text_part_ids.take() {
            self.text_part_ids = OnceLock::from(text_part_ids);
        }

        if let Some(html_part_ids) = other.html_part_ids.take() {
            self.html_part_ids = OnceLock::from(html_part_ids);
        }

        if let Some(body_parts) = other.body_parts.take() {
            self.body_parts = OnceLock::from(body_parts);
        }
    }
}

impl From<jmap_client::email::Email> for CachedMail {
    fn from(mut jmap_mail: jmap_client::email::Email) -> Self {
        let body_parts = {
            let mut body_parts = HashMap::new();

            if let Some(text_body_parts) = jmap_mail.text_body() {
                for text_body_part in text_body_parts {
                    let part_id = text_body_part.part_id().unwrap().to_string();
                    body_parts.insert(
                        part_id.clone(),
                        jmap_mail
                            .body_value(part_id.as_str())
                            .unwrap()
                            .value()
                            .to_string(),
                    );
                }
            }

            if let Some(html_body_parts) = jmap_mail.html_body() {
                for html_body_part in html_body_parts {
                    let part_id = html_body_part.part_id().unwrap().to_string();
                    body_parts.insert(
                        part_id.clone(),
                        jmap_mail
                            .body_value(part_id.as_str())
                            .unwrap()
                            .value()
                            .to_string(),
                    );
                }
            }

            OnceLock::from(body_parts)
        };

        Self {
            id: jmap_mail.take_id().map(MailId::from).unwrap(),
            keywords: jmap_mail.keywords().map_or_default(|keywords| {
                let keywords = keywords
                    .into_iter()
                    .map(MailKeyword::from)
                    .collect::<HashSet<MailKeyword>>();

                OnceLock::from(keywords)
            }),
            mailbox_ids: jmap_mail.mailbox_ids().map_or_default(|mailbox_ids| {
                let ids = mailbox_ids
                    .into_iter()
                    .map(MailboxId::from)
                    .collect::<Vec<MailboxId>>();

                OnceLock::from(ids)
            }),
            subject: match jmap_mail.subject() {
                Some(subject) => OnceLock::from(subject.to_string()),
                None => OnceLock::new(),
            },
            received_at: jmap_mail.received_at().map_or_default(|received_at| {
                let received_at = DateTime::<Utc>::from_timestamp(received_at, 0)
                    .map(|time| time.with_timezone(&Local))
                    .unwrap();

                OnceLock::from(received_at)
            }),
            has_attachment: jmap_mail.has_attachment().map_or_default(OnceLock::from),
            thread_id: jmap_mail.thread_id().map_or_default(|id| {
                let id = ThreadId::from(id);
                OnceLock::from(id)
            }),
            from: match jmap_mail.from() {
                Some(from) => {
                    let addresses = MailAddresses::from(from);
                    OnceLock::from(addresses)
                }
                None => OnceLock::new(),
            },
            to: match jmap_mail.to() {
                Some(to) => {
                    let addresses = MailAddresses::from(to);
                    OnceLock::from(addresses)
                }
                None => OnceLock::new(),
            },
            cc: match jmap_mail.cc() {
                Some(cc) => {
                    let addresses = MailAddresses::from(cc);
                    OnceLock::from(addresses)
                }
                None => OnceLock::new(),
            },
            bcc: match jmap_mail.bcc() {
                Some(bcc) => {
                    let addresses = MailAddresses::from(bcc);
                    OnceLock::from(addresses)
                }
                None => OnceLock::new(),
            },
            preview: jmap_mail.take_preview().map_or_default(OnceLock::from),
            attachments: jmap_mail.attachments().map_or_default(|attachments| {
                let attachments: Vec<MailDataAttachment> =
                    attachments.iter().map(MailDataAttachment::from).collect();
                OnceLock::from(attachments)
            }),
            text_part_ids: jmap_mail.text_body().map_or_default(|parts| {
                let parts: Vec<String> = parts
                    .into_iter()
                    .map(|part| part.part_id().unwrap().to_string())
                    .collect();

                OnceLock::from(parts)
            }),
            html_part_ids: jmap_mail.html_body().map_or_default(|parts| {
                let parts: Vec<String> = parts
                    .into_iter()
                    .map(|part| part.part_id().unwrap().to_string())
                    .collect();

                OnceLock::from(parts)
            }),
            body_parts,
        }
    }
}
