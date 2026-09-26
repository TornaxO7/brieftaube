use crate::types::{MailAddresses, MailDataAttachment, MailId};
use chrono::{DateTime, Local, Utc};
use jmap_client::email::{Email, Property};

#[derive(Debug, Clone)]
pub struct MailDataPreview {
    pub id: MailId,
    pub from: Option<MailAddresses>,
    pub to: Option<MailAddresses>,
    pub cc: Option<MailAddresses>,
    pub bcc: Option<MailAddresses>,
    pub subject: Option<String>,
    pub preview: Option<String>,
    pub attachments: Option<Vec<MailDataAttachment>>,
    pub received_at: DateTime<Local>,
}

impl MailDataPreview {
    pub const GET_REQUEST_PROPERTIES: [Property; 9] = [
        Property::Id,
        Property::From,
        Property::To,
        Property::Cc,
        Property::Bcc,
        Property::Subject,
        Property::Preview,
        Property::Attachments,
        Property::ReceivedAt,
    ];

    pub fn from_get_request(mut mail: Email) -> Self {
        Self {
            id: mail.take_id().into(),
            from: mail.take_from().map(MailAddresses::from),
            to: mail.take_to().map(MailAddresses::from),
            cc: mail.take_cc().map(MailAddresses::from),
            bcc: mail.take_bcc().map(MailAddresses::from),
            subject: mail.take_subject(),
            preview: mail.take_preview(),
            attachments: mail
                .attachments()
                .map(|parts| parts.iter().map(MailDataAttachment::from).collect()),
            received_at: DateTime::<Utc>::from_timestamp(mail.received_at().unwrap(), 0)
                .expect("Valid timestamp")
                .with_timezone(&Local),
        }
    }
}
