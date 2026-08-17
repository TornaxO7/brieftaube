use std::{collections::HashSet, sync::Arc};

use super::MailDisplayAttachment;
use crate::backend::{
    Backend,
    mails::types::{
        MailData, MailDataHtmlBody, MailDataTextBody, MailKeyword, addresses_to_string,
    },
};

pub struct MailDisplay {
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub received_at: String,
    pub keywords: String,

    pub html_body: Option<MailDataHtmlBody>,
    pub text_body: Option<MailDataTextBody>,
    pub attachments: Option<Vec<MailDisplayAttachment>>,
}

impl MailDisplay {
    pub fn new(mail: MailData, backend: Arc<Backend>) -> Self {
        Self {
            from: addresses_to_string(&mail.from),
            to: addresses_to_string(&mail.to),
            cc: addresses_to_string(&mail.cc),
            subject: mail.subject,
            received_at: mail.received_at.format("%A, %d %B %Y %T").to_string(),

            keywords: convert_keywords_to_string(&mail.keywords),
            html_body: mail.html_body,
            text_body: mail.text_body,
            attachments: mail.attachments.map(|attachments| {
                attachments
                    .into_iter()
                    .map(|attachment| MailDisplayAttachment::new(attachment, backend.clone()))
                    .collect()
            }),
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
