use super::ScrollableBody;
use crate::{types::MailDataHtmlBody, ui::pager::body::BodyReader};

#[derive(Debug)]
pub struct HtmlBody {
    pub html: String,
    pub state: std::io::Result<ScrollableBody>,
}

impl HtmlBody {
    pub fn new(html: MailDataHtmlBody) -> Option<Self> {
        let content = html.content?;
        let ctx = htmd::convert(content.as_str()).map(ScrollableBody::new);

        Some(Self {
            html: content,
            state: ctx,
        })
    }
}

impl BodyReader for HtmlBody {
    fn scrollable_body(&mut self) -> Option<&mut ScrollableBody> {
        self.state.as_mut().ok()
    }
}
