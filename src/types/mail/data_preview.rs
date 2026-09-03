use crate::types::{MailAddresses, MailDataAttachment};
use jmap_client::email::{Email, Property};

#[derive(Debug, Clone)]
pub struct MailDataPreview {
    pub from: Option<MailAddresses>,
    pub to: Option<MailAddresses>,
    pub cc: Option<MailAddresses>,
    pub bcc: Option<MailAddresses>,
    pub preview: Option<String>,
    pub attachments: Option<Vec<MailDataAttachment>>,
}

impl MailDataPreview {
    pub const GET_REQUEST_PROPERTIES: [Property; 7] = [
        Property::Id,
        Property::From,
        Property::To,
        Property::Cc,
        Property::Bcc,
        Property::Preview,
        Property::Attachments,
    ];

    pub fn from_get_request(mut mail: Email) -> Self {
        Self {
            from: mail.take_from().map(MailAddresses::from),
            to: mail.take_to().map(MailAddresses::from),
            cc: mail.take_cc().map(MailAddresses::from),
            bcc: mail.take_bcc().map(MailAddresses::from),
            preview: mail.take_preview(),
            attachments: mail
                .attachments()
                .map(|parts| parts.iter().map(MailDataAttachment::from).collect()),
        }
    }
}
