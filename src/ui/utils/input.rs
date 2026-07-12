use crate::ui::ScreenOverlayResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use ratatui_textarea::TextArea;

pub struct State {
    input: TextArea<'static>,
    desc: String,
}

impl State {
    pub fn new<S: ToString>(desc: S) -> Self {
        Self {
            input: TextArea::default(),
            desc: desc.to_string(),
        }
    }

    pub fn handle_event<P>(&mut self, event: KeyEvent) -> Option<ScreenOverlayResult<P>> {
        match event.code {
            KeyCode::Esc => Some(ScreenOverlayResult::Cancel),
            KeyCode::Enter => Some(ScreenOverlayResult::Input(
                self.input.lines().get(0).unwrap().to_owned(),
            )),
            _ => {
                self.input.input(event);
                None
            }
        }
    }
}

#[derive(Default)]
pub struct Input;

impl StatefulWidget for Input {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered();
        let [top, bottom] = Layout::vertical([Constraint::Length(2), Constraint::Length(3)])
            .areas(block.inner(area));

        Widget::render(block, area, buf);
        Widget::render(
            Paragraph::new(state.desc.as_str()),
            Block::default().inner(top),
            buf,
        );

        state.input.set_block(Block::bordered());
        state.input.render(bottom, buf);
    }
}
