use crate::types::{MailAddresses, MailDataAttachment, MailDto, MailId};
use chrono::{DateTime, Local};
use jmap_client::email::Property;

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
    pub received_at: Option<DateTime<Local>>,
}

impl MailDataPreview {
    pub fn new(cached_mail: MailDto) -> Option<Self> {
        let core = cached_mail.core;
        let preview = cached_mail.preview?;

        Some(Self {
            id: core.id,
            from: preview.from,
            to: preview.to,
            cc: preview.cc,
            bcc: preview.bcc,
            subject: core.subject,
            preview: preview.preview,
            attachments: preview.attachments,
            received_at: core.received_at,
        })
    }
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
}
