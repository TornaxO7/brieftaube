use std::{collections::HashSet, sync::Arc};

use super::MailDisplayAttachment;
use crate::backend::{
    Backend, MailId,
    mails::types::{
        MailData, MailDataHtmlBody, MailDataTextBody, MailKeyword, addresses_to_string,
    },
    types::RemoteData,
};

pub struct MailDisplay {
    pub id: MailId,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub received_at: String,
    pub keywords: String,

    pub html_body: RemoteData<MailDataHtmlBody>,
    pub text_body: RemoteData<MailDataTextBody>,
    pub attachments: RemoteData<Vec<MailDisplayAttachment>>,
}

impl MailDisplay {
    pub fn new(mail: MailData, backend: Arc<Backend>) -> Self {
        Self {
            id: mail.id,
            from: addresses_to_string(&mail.from),
            to: addresses_to_string(&mail.to),
            cc: addresses_to_string(&mail.cc),
            subject: mail.subject,
            received_at: mail.received_at.format("%A, %d %B %Y %T").to_string(),

            keywords: convert_keywords_to_string(&mail.keywords),
            html_body: mail.html_body,
            text_body: mail.text_body,
            attachments: {
                mail.attachments.map(|attachments| {
                    attachments
                        .into_iter()
                        .map(|attachment| MailDisplayAttachment::new(attachment, backend.clone()))
                        .collect()
                })
            },
        }
    }
}

fn convert_keywords_to_string(keywords: &HashSet<MailKeyword>) -> String {
    keywords
        .iter()
        .map(|keyword| keyword.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}
