use crate::{types::CachedMail, ui::pager::body::BodyReader};
use pulldown_cmark_mdcat::ratatui::MdcatWidgetState;

pub struct HtmlBody {
    pub html: String,

    pub ctx: std::io::Result<Ctx>,
}

impl From<&CachedMail> for HtmlBody {
    fn from(mail: &CachedMail) -> Self {
        let html = mail.html_body_as_string().unwrap();
        let ctx = htmd::convert(html.as_str()).map(|markdown| Ctx {
            markdown,
            state: MdcatWidgetState::new(),
        });

        Self { html, ctx }
    }
}

impl BodyReader for HtmlBody {
    fn navigate_up(&mut self, amount: usize) {
        if let Ok(ctx) = self.ctx.as_mut() {
            ctx.state.scroll_up(amount as u16);
        }
    }

    fn navigate_down(&mut self, amount: usize) {
        if let Ok(ctx) = self.ctx.as_mut() {
            ctx.state.scroll_down(amount as u16);
        }
    }

    fn navigate_to_top(&mut self) {
        if let Ok(ctx) = self.ctx.as_mut() {
            ctx.state.set_scroll(0);
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Ok(ctx) = self.ctx.as_mut() {
            ctx.state.set_scroll(ctx.state.total_lines() as u16);
        }
    }

    fn navigate_half_page_up(&mut self) {
        if let Ok(ctx) = self.ctx.as_mut() {
            ctx.state.scroll_up(10);
        }
    }

    fn navigate_half_page_down(&mut self) {
        if let Ok(ctx) = self.ctx.as_mut() {
            ctx.state.scroll_down(10);
        }
    }
}

pub struct Ctx {
    pub markdown: String,
    pub state: MdcatWidgetState,
}
