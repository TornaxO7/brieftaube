use crate::types::MailDto;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataTextBody {
    pub content: String,
}

impl MailDataTextBody {
    pub fn new(cached_mail: MailDto) -> Option<Self> {
        let text_part_ids = cached_mail.text_part_ids?;
        let content = text_part_ids
            .0
            .iter()
            .map(|id| cached_mail.body_parts.get(id).unwrap())
            .fold(String::new(), |prev, next| format!("{}{}", prev, next));

        Some(Self { content })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataHtmlBody {
    pub content: String,
}

impl MailDataHtmlBody {
    pub fn new(cached_mail: MailDto) -> Option<Self> {
        let html_part_ids = cached_mail.html_part_ids?;
        let content = html_part_ids
            .0
            .iter()
            .map(|id| cached_mail.body_parts.get(id).unwrap())
            .fold(String::new(), |prev, next| format!("{}{}", prev, next));

        Some(Self { content })
    }
}

// fn join_body_values(mail: &Email, parts: &[EmailBodyPart]) -> Option<String> {
//     let mut body = String::new();

//     for part in parts {
//         let Some(part_id) = part.part_id() else {
//             continue;
//         };

//         if let Some(value) = mail.body_value(part_id) {
//             body.push_str(value.value());
//         }
//     }

//     if body.is_empty() { None } else { Some(body) }
// }
