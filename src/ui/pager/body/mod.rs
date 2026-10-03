mod html;
mod text;

use ratatui::{
    layout::Size,
    widgets::{Paragraph, Wrap},
};

pub use html::*;
pub use text::*;

pub trait BodyReader {
    fn scrollable_body(&mut self) -> Option<&mut ScrollableBody>;

    fn navigate_up(&mut self, amount: usize) {
        if let Some(state) = self.scrollable_body() {
            state.navigate_down(amount);
        }
    }

    fn navigate_down(&mut self, amount: usize) {
        if let Some(state) = self.scrollable_body() {
            state.navigate_down(amount);
        }
    }

    fn navigate_to_top(&mut self) {
        if let Some(state) = self.scrollable_body() {
            state.navigate_to_top();
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Some(state) = self.scrollable_body() {
            state.navigate_to_bottom();
        }
    }

    fn navigate_half_page_up(&mut self) {
        if let Some(state) = self.scrollable_body() {
            state.navigate_half_page_up();
        }
    }

    fn navigate_half_page_down(&mut self) {
        if let Some(state) = self.scrollable_body() {
            state.navigate_half_page_down();
        }
    }
}

#[derive(Debug)]
pub struct ScrollableBody {
    pub content: String,
    pub scroll_offset: usize,
    pub content_height: usize,
}

impl ScrollableBody {
    fn new(content: String) -> Self {
        let content_height = content.lines().count();

        Self {
            content,
            scroll_offset: 0,
            content_height,
        }
    }

    pub fn default_paragraph<'a>(&'a self) -> Paragraph<'a> {
        Paragraph::new(self.content.as_str()).wrap(Wrap { trim: false })
    }

    pub fn set_content_height(&mut self, size: Size) {
        let amount_lines = self.default_paragraph().line_count(size.width);

        // for whatever reason there seems to be an off-by-one-error sometimes in the calculation.
        // So just to be sure, we are assuming an even smaller display size
        let body_area_height = size.height.saturating_sub(1);

        self.content_height = amount_lines.saturating_sub(body_area_height as usize);
    }

    fn navigate_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    fn navigate_down(&mut self, amount: usize) {
        self.scroll_offset = self
            .scroll_offset
            .saturating_add(amount)
            .min(self.content_height);
    }

    fn navigate_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    fn navigate_to_bottom(&mut self) {
        self.scroll_offset = self.content_height;
    }

    fn navigate_half_page_down(&mut self) {
        self.navigate_down(self.content_height / 2);
    }

    fn navigate_half_page_up(&mut self) {
        self.navigate_up(self.content_height / 2);
    }
}
