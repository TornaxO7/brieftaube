use crate::ui::reader::body::{BodyReader, ScrollableBody};

#[derive(Debug)]
pub struct TextBody {
    pub state: ScrollableBody,
}

impl TextBody {
    pub fn new(content: String) -> Self {
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
