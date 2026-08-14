use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, StatefulWidget, Widget},
};

pub struct Prompt;

impl StatefulWidget for Prompt {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state
            .input
            .set_block(Block::bordered().title(state.desc.clone()));

        state.input.render(area, buf);
    }
}
