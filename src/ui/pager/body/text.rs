use crate::{
    types::MailDataTextBody,
    ui::pager::body::{BodyReader, ScrollableBody},
};

#[derive(Debug)]
pub struct TextBody {
    pub state: ScrollableBody,
}

impl TextBody {
    pub fn new(text: MailDataTextBody) -> Self {
        Self {
            state: ScrollableBody::new(text.content),
        }
    }
}

impl BodyReader for TextBody {
    fn scrollable_body(&mut self) -> Option<&mut ScrollableBody> {
        Some(&mut self.state)
    }
}
