use std::marker::PhantomData;

use crate::utils::ui::ScreenOverlayResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use ratatui_textarea::TextArea;

pub struct State<I> {
    input: TextArea<'static>,
    desc: String,
    typ: I,
}

impl<I: Clone> State<I> {
    pub fn new<S: ToString>(desc: S, typ: I) -> Self {
        Self {
            input: TextArea::default(),
            desc: desc.to_string(),
            typ,
        }
    }

    pub fn handle_event<P>(&mut self, event: KeyEvent) -> Option<ScreenOverlayResult<P, I>> {
        match event.code {
            KeyCode::Esc => Some(ScreenOverlayResult::Cancel),
            KeyCode::Enter => Some(ScreenOverlayResult::Input {
                value: self.input.lines().get(0).unwrap().to_owned(),
                typ: self.typ.clone(),
            }),
            _ => {
                self.input.input(event);
                None
            }
        }
    }
}

pub struct Input<I> {
    _phantom: PhantomData<I>,
}

impl<I> Input<I> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<I> StatefulWidget for Input<I> {
    type State = State<I>;

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
