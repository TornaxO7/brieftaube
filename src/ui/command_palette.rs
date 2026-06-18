use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{List, ListDirection, ListState, Paragraph, StatefulWidget, Widget},
};
use ratatui_textarea::TextArea;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
}

impl Command {
    pub fn new<S: ToString>(name: S, description: S) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HandleEventResult {
    Selected(usize),
    Quit,
}

#[derive(Debug, Default)]
pub struct State {
    input: TextArea<'static>,
    commands: Vec<Command>,
}

impl State {
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            commands,
            ..Default::default()
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<HandleEventResult> {
        match event.code {
            KeyCode::Esc => return Some(HandleEventResult::Quit),
            _ => {}
        }

        self.input.input(event);

        None
    }

    pub fn reset(&mut self) {
        self.input.clear();
    }
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [left, description] = area.layout(
            &Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)]),
        );

        let [search, options] = left.layout(
            &Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(90)]),
        );

        Widget::render(Paragraph::new("This is a description"), description, buf);

        self.input.render(search, buf);

        StatefulWidget::render(
            List::new(["Result 1", "Result 2"])
                .highlight_style(Style::new().blue())
                .direction(ListDirection::BottomToTop),
            options,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        );
    }
}
