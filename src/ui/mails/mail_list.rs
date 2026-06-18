use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget, Widget},
};

use crate::ui::Mode;

#[derive(Debug)]
pub struct State {
    is_focussed: bool,
}

impl State {
    pub fn new() -> Self {
        Self { is_focussed: true }
    }

    pub fn set_focus(&mut self, is_focussed: bool) {
        self.is_focussed = is_focussed;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn handle_event(&mut self, event: crossterm::event::KeyEvent) {}
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = {
            let mut block = Block::bordered().title("Mails");

            block = if self.is_focussed {
                block.border_style(Style::new().green())
            } else {
                block
            };

            block
        };

        StatefulWidget::render(
            List::new(["Mail 1", "Mail 2", "Mail 3"])
                .block(block)
                .highlight_style(Style::new().blue())
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        )
    }
}
