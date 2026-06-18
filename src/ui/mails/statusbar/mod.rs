use ratatui::{
    layout::{Constraint, Direction, Flex, HorizontalAlignment, Layout},
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct State {
    focus_name: &'static str,
}

impl State {
    pub fn new(init_name: &'static str) -> Self {
        Self {
            focus_name: init_name,
        }
    }
}

impl State {
    pub fn set_focus_name(&mut self, name: &'static str) {
        self.focus_name = name;
    }
}

impl Widget for &State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [left, center, right] = area.layout(
            &Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::SpaceBetween)
                .constraints([
                    Constraint::Fill(0),
                    Constraint::Fill(0),
                    Constraint::Fill(0),
                ]),
        );

        // Widget::render(Block::bordered().title("Statusbar"), area, buf);

        // left
        Widget::render(
            Paragraph::new(self.focus_name).alignment(HorizontalAlignment::Left),
            left.centered_vertically(Constraint::Min(0)),
            buf,
        );

        // center
        Widget::render(
            Paragraph::new(self.focus_name).alignment(HorizontalAlignment::Center),
            center.centered_vertically(Constraint::Min(0)),
            buf,
        );

        // right
        Widget::render(
            Paragraph::new(self.focus_name).alignment(HorizontalAlignment::Right),
            right.centered_vertically(Constraint::Min(1)),
            buf,
        );
    }
}
