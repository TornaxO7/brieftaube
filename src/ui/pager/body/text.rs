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
    fn navigate_up(&mut self, amount: usize) {
        self.state.navigate_up(amount);
    }

    fn navigate_down(&mut self, amount: usize) {
        self.state.navigate_down(amount);
    }

    fn navigate_to_top(&mut self) {
        self.state.navigate_to_top();
    }

    fn navigate_to_bottom(&mut self) {
        self.state.navigate_to_bottom();
    }

    fn navigate_half_page_up(&mut self) {
        self.state.navigate_up(self.state.content_height / 2);
    }

    fn navigate_half_page_down(&mut self) {
        self.state.navigate_down(self.state.content_height / 2);
    }
}
