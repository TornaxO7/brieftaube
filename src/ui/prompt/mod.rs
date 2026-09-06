mod view;

use crate::ui::Layer;
use crossterm::event::{Event, KeyCode};
use ratatui::style::Style;
use ratatui_textarea::TextArea;

pub use view::view;

pub enum Message {
    Reset(String),
    Event(Event),
}

pub struct State {
    pub input: TextArea<'static>,
    pub desc: String,
}

impl State {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());

        Self {
            input,
            desc: String::new(),
        }
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Option<super::Message> {
        match msg {
            Message::Reset(desc) => self.handle_reset(desc),
            Message::Event(event) => self.handle_event(event),
        }
    }
}

impl State {
    fn handle_reset(&mut self, desc: String) -> Option<super::Message> {
        self.desc = desc;
        self.input.clear();
        None
    }

    fn handle_event(&mut self, event: Event) -> Option<super::Message> {
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Enter => Some(super::Message::Back),
                KeyCode::Esc => {
                    self.input.clear();
                    Some(super::Message::Back)
                }
                _ => {
                    self.input.input(event);
                    None
                }
            },
            _ => None,
        }
    }
}
