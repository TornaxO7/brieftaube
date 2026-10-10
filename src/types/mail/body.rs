use crate::types::CachedMail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataTextBody {
    pub content: String,
}

impl MailDataTextBody {
    pub fn new(cached_mail: CachedMail) -> Option<Self> {
        let body_parts = cached_mail.body_parts.into_inner().unwrap();
        let part_ids = cached_mail.text_part_ids.into_inner().unwrap();

        let content = part_ids
            .into_iter()
            .fold(String::new(), |prev, next_part_id| {
                let next_part = body_parts.get(&next_part_id).unwrap();
                format!("{}{}", prev, next_part)
            });

        Some(Self { content })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataHtmlBody {
    pub content: String,
}

impl MailDataHtmlBody {
    pub fn new(cached_mail: CachedMail) -> Option<Self> {
        let body_parts = cached_mail.body_parts.into_inner().unwrap();
        let part_ids = cached_mail.html_part_ids.into_inner().unwrap();

        let content = part_ids
            .into_iter()
            .fold(String::new(), |prev, next_part_id| {
                let next_part = body_parts.get(&next_part_id).unwrap();
                format!("{}{}", prev, next_part)
            });

        Some(Self { content })
    }
}
