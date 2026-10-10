// use super::MailKeyword;
// use crate::types::{CachedMail, MailId, MailboxId, Missing, ThreadId};
// use chrono::{DateTime, Local};
// use jmap_client::email::Property;
// use std::collections::HashSet;

// #[derive(Debug, Clone)]
// pub struct MailDataCore {
//     pub id: MailId,
//     pub keywords: HashSet<MailKeyword>,
//     pub subject: Option<String>,
//     pub received_at: DateTime<Local>,
//     pub has_attachment: bool,
//     pub mailbox_ids: Vec<MailboxId>,
//     pub thread_id: ThreadId,
// }

// impl MailDataCore {
//     pub const GET_REQUEST_PROPERTIES: [Property; 6] = [
//         Property::Keywords,
//         Property::Subject,
//         Property::ReceivedAt,
//         Property::HasAttachment,
//         Property::MailboxIds,
//         Property::ThreadId,
//     ];
// }

// impl TryFrom<CachedMail> for MailDataCore {
//     type Error = Missing;

//     fn try_from(mail: CachedMail) -> Result<Self, Self::Error> {
//         let id = mail.id;
//         let keywords = mail.keywords;
//         let subject = mail.subject.into_inner().ok_or(Missing)?;
//         let received_at = mail.received_at.into_inner().ok_or(Missing)?;
//         let has_attachment = mail.has_attachment;
//         let mailbox_ids = mail.mailbox_ids;
//         let thread_id = mail.thread_id.into_inner().ok_or(Missing)?;

//         Ok(Self {
//             id,
//             keyword,
//             subject,
//             received_at,
//             has_attachment,
//             mailbox_ids,
//             thread_id,
//         })
//     }
// }

// // impl From<CachedMail> for MailDataCore {
// //     fn from(cached_mail: CachedMail) -> Self {
// //         let stage0 = cached_mail.core;

// //         Self {
// //             id: stage0.id,
// //             keywords: stage0.keywords,
// //             subject: stage0.subject,
// //             received_at: stage0.received_at.with_timezone(&Local),
// //             has_attachment: stage0.has_attachment,
// //             mailbox_ids: stage0.mailbox_ids,
// //             thread_id: stage0.thread_id,
// //         }
// //     }
// // }
