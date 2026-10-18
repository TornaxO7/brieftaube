use super::ScrollableBody;
use crate::{types::CachedMail, ui::pager::body::BodyReader};

#[derive(Debug)]
pub struct HtmlBody {
    pub html: String,
    pub state: std::io::Result<ScrollableBody>,
}

impl From<&CachedMail> for HtmlBody {
    fn from(mail: &CachedMail) -> Self {
        let html = mail.html_body_as_string().unwrap();
        let state = htmd::convert(html.as_str()).map(ScrollableBody::new);

        Self { html, state }
    }
}

impl BodyReader for HtmlBody {
    fn scrollable_body(&mut self) -> Option<&mut ScrollableBody> {
        self.state.as_mut().ok()
    }
}
