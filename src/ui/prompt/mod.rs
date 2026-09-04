mod view;

use crate::ui::{Action, LayerCore, LayerMessage};
use crossterm::event::{Event, KeyCode};
use ratatui::style::Style;
use ratatui_textarea::TextArea;

pub use view::view;

pub struct State {
    pub input: TextArea<'static>,
    pub desc: String,
}

impl State {
    pub fn new<S: ToString>(desc: S) -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());

        Self {
            input,
            desc: desc.to_string(),
        }
    }
}

impl From<State> for Option<LayerMessage> {
    fn from(state: State) -> Self {
        state.input.lines().first().cloned().map(LayerMessage::from)
    }
}

impl LayerCore for State {
    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Enter => Some(Action::Back),
                KeyCode::Esc => {
                    self.input.clear();
                    Some(Action::Back)
                }
                _ => {
                    self.input.input(event);
                    None
                }
            },
            _ => None,
        }
    }

    fn handle_layer_message<Msg>(&mut self, _: Msg) -> Option<Action>
    where
        Msg: Into<Option<super::LayerMessage>>,
    {
        None
    }
}
