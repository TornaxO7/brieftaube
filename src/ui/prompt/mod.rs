mod view;

use crate::ui::Layer;
use crossterm::event::{Event, KeyCode};
use ratatui::style::Style;
use ratatui_textarea::TextArea;

pub use view::view;

#[derive(Debug)]
pub enum Message {
    Event(Event),
}

pub struct State {
    pub input: TextArea<'static>,
    pub description: String,

    map: Box<dyn Fn(String) -> super::Message>,
}

impl State {
    pub fn new(description: String, map: Box<dyn Fn(String) -> super::Message>) -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());

        Self {
            input,
            map,
            description,
        }
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        match msg {
            Message::Event(event) => self.handle_event(event),
        }
    }
}

impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Enter => {
                    let input = self.input.lines()[0].clone();
                    let msg = (self.map)(input);
                    vec![super::Message::Back, msg]
                }
                KeyCode::Esc => {
                    self.input.clear();
                    vec![super::Message::Back]
                }
                _ => {
                    self.input.input(event);
                    vec![]
                }
            },
            _ => vec![],
        }
    }
}
