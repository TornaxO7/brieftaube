mod list;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{
    backend::mailboxes::MailboxData,
    ui::{ScreenOverlay, ScreenState, utils},
};

#[derive(Default)]
pub struct Mailboxes {}

impl StatefulWidget for Mailboxes {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(layers) = &mut state.layers {
            if let Some(parent_layer) = layers.parent_layer_render_data() {
                render_layer(area, buf, parent_layer);
            }

            render_layer(area, buf, layers.current_layer_render_data());

            if let Some(children_layer) = layers.current_selected_children_mailboxes() {
                render_layer(area, buf, children_layer);
            }
        } else {
            render_loading_screen(area, buf);
        }

        render_overlay(area, buf, state);
    }
}

fn render_layer(
    area: Rect,
    buf: &mut Buffer,
    render_data: (&[MailboxData], &mut tui_widget_list::ListState),
) {
    StatefulWidget::render(
        list::List::new(render_data.0).block(
            Block::new()
                .title("Mailboxes")
                .title_alignment(HorizontalAlignment::Center),
        ),
        area,
        buf,
        render_data.1,
    )
}

fn render_loading_screen(area: Rect, buf: &mut Buffer) {
    Widget::render(
        Paragraph::new("Loading mailboxes...")
            .block(Block::bordered())
            .centered(),
        area,
        buf,
    );
}

fn render_overlay(area: Rect, buf: &mut Buffer, state: &mut super::State) {
    if let Some(state) = state.overlay() {
        match state {
            ScreenOverlay::Palette(state) => {
                let area = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
                Widget::render(Clear, area, buf);
                StatefulWidget::render(utils::palette::Palette::new(), area, buf, state);
            }
            ScreenOverlay::Input(state) => {
                let area = area.centered(Constraint::Percentage(20), Constraint::Percentage(15));
                Widget::render(Clear, area, buf);
                StatefulWidget::render(utils::input::Input::new(), area, buf, state)
            }
        }
    }
}
