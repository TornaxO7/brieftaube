use crate::{
    types::CachedMail,
    ui::pager::body::{BodyReader, ScrollableBody},
};

#[derive(Debug)]
pub struct TextBody {
    pub state: ScrollableBody,
}

impl From<&CachedMail> for TextBody {
    fn from(mail: &CachedMail) -> Self {
        let content = mail.text_body_as_string().unwrap();

        Self {
            state: ScrollableBody::new(content),
        }
    }
}

impl BodyReader for TextBody {
    fn scrollable_body(&mut self) -> Option<&mut ScrollableBody> {
        Some(&mut self.state)
    }
}
