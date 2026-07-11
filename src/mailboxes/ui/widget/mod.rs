mod list;

use ratatui::{
    layout::HorizontalAlignment,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct Mailboxes {}

impl StatefulWidget for Mailboxes {
    type State = super::State;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        if let Some(mailboxes) = &state.mailboxes {
            StatefulWidget::render(
                list::List::new(&mailboxes).block(
                    Block::new()
                        .title("Mailboxes")
                        .title_alignment(HorizontalAlignment::Center),
                ),
                area,
                buf,
                &mut state.list_state,
            )
        } else {
            Widget::render(
                Paragraph::new("Loading mailboxes...")
                    .block(Block::bordered())
                    .centered(),
                area,
                buf,
            );
        }
    }
}
