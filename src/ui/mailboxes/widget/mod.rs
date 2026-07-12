mod list;

use ratatui::{
    layout::{Constraint, HorizontalAlignment},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::ui::{ScreenOverlay, ScreenState, utils};

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

        if let Some(state) = state.overlay() {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Widget::render(Clear, a, buf);
            match state {
                ScreenOverlay::Palette(state) => {
                    StatefulWidget::render(utils::palette::Palette::new(), a, buf, state);
                }
                ScreenOverlay::Input(state) => {
                    StatefulWidget::render(utils::input::Input::default(), a, buf, state)
                }
            }
        }
    }
}
