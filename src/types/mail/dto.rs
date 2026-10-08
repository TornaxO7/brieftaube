use crate::types::{MailAddresses, MailDataAttachment, MailId, MailKeyword, MailboxId, ThreadId};
use chrono::{DateTime, Local, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MailDto {
    pub core: MailDtoCore,
    pub preview: Option<MailDtoPreview>,

    pub text_part_ids: Option<Vec<String>>,
    pub html_part_ids: Option<Vec<String>>,
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

// impl From<jmap_client::email::Email> for MailDto {
//     fn from(mut jmap_mail: jmap_client::email::Email) -> Self {
//         let body_parts = {
//             let mut body_parts = HashMap::new();

//             if let Some(text_body_parts) = jmap_mail.text_body() {
//                 for body_part in text_body_parts {
//                     let Some(part_id) = body_part.part_id() else {
//                         continue;
//                     };

//                     body_parts.insert(
//                         part_id.to_string(),
//                         jmap_mail.body_value(part_id).unwrap().value().to_string(),
//                     );
//                 }
//             }

//             if let Some(html_body_parts) = jmap_mail.html_body() {
//                 for body_part in html_body_parts {
//                     let Some(part_id) = body_part.part_id() else {
//                         continue;
//                     };

//                     body_parts.insert(
//                         part_id.to_string(),
//                         jmap_mail.body_value(part_id).unwrap().value().to_string(),
//                     );
//                 }
//             }

//             body_parts
//         };

//         Self {

//             stage1:

//             text_part_ids: jmap_mail.text_body().map(|body_parts| {
//                 body_parts
//                     .iter()
//                     .map(|body_part| body_part.part_id().map(|id| id.to_string()))
//                     .flatten()
//                     .collect()
//             }),
//             html_part_ids: jmap_mail.html_body().map(|body_parts| {
//                 body_parts
//                     .iter()
//                     .map(|body_part| body_part.part_id().map(|id| id.to_string()))
//                     .flatten()
//                     .collect()
//             }),
//             body_parts,
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct MailDtoCore {
    pub id: MailId,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: Option<DateTime<Local>>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
    pub thread_id: Option<ThreadId>,
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
                .map(|time| time.with_timezone(&Local)),
            has_attachment: jmap_mail.has_attachment(),
            mailbox_ids: jmap_mail
                .mailbox_ids()
                .into_iter()
                .map(|id| MailboxId(id.to_string()))
                .collect(),
            thread_id: jmap_mail.take_thread_id().map(ThreadId::from),
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
