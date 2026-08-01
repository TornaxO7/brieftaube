use std::collections::HashSet;

use crate::backend::mails::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailKeyword,
    addresses_to_string,
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

impl From<MailData> for MailDisplay {
    fn from(mail: MailData) -> Self {
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
                    .map(MailDisplayAttachment::from)
                    .collect()
            }),
        }
    }
}

pub struct MailDisplayAttachment {
    pub name: String,
    pub content_type: String,
    pub size: String,
}

impl From<MailDataAttachment> for MailDisplayAttachment {
    fn from(attachment: MailDataAttachment) -> Self {
        let size = {
            const KB: f64 = 1024.0;
            const MB: f64 = KB * 1024.0;
            const GB: f64 = MB * 1024.0;

            let size = attachment.size as f64;
            if size >= GB {
                format!("{:.1}G", size / GB)
            } else if size >= MB {
                format!("{:.1}M", size / MB)
            } else if size >= KB {
                format!("{:.1}K", size / KB)
            } else {
                format!("{}B", attachment.size)
            }
        };

        Self {
            name: attachment.name,
            content_type: attachment.content_type,
            size,
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
