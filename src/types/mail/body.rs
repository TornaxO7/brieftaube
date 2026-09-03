use jmap_client::email::{Email, EmailBodyPart};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataTextBody(pub String);

impl MailDataTextBody {
    pub fn new(mail: &Email) -> Option<Self> {
        let parts = mail.text_body()?;
        let content = join_body_values(mail, parts)?;

        Some(Self(content))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDataHtmlBody(pub String);

impl MailDataHtmlBody {
    pub fn new(mail: &Email) -> Option<Self> {
        let parts = mail.html_body()?;
        let content = join_body_values(mail, parts)?;

        Some(Self(content))
    }
}
fn join_body_values(mail: &Email, parts: &[EmailBodyPart]) -> Option<String> {
    let mut body = String::new();

    for part in parts {
        let Some(part_id) = part.part_id() else {
            continue;
        };

        if let Some(value) = mail.body_value(part_id) {
            body.push_str(value.value());
        }
    }

    if body.is_empty() { None } else { Some(body) }
}
