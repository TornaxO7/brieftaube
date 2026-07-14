mod list;

use crate::ui::{ScreenOverlay, ScreenState, mailboxes::state::Layer, utils};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Table, Widget},
};

#[derive(Default)]
pub struct Mailboxes {}

impl StatefulWidget for Mailboxes {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(layers) = &mut state.layers {
            let [left, center, right] = Layout::horizontal([
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
            ])
            .areas(area);

            if let Some(parent_layer) = layers.get_parent_layer_mut() {
                render_layer(left, buf, parent_layer);
            }

            render_layer(center, buf, layers.get_current_layer_mut());

            render_layer(right, buf, layers.get_children_layer_mut());
        } else {
            render_loading_screen(area, buf);
        }

        render_overlay(area, buf, state);
    }
}

fn render_layer(area: Rect, buf: &mut Buffer, layer: &mut Layer) {
    let table = {
        let rows = {
            let mut rows = Vec::with_capacity(layer.mailboxes.capacity() + 1);

            if !layer.is_root_layer() {
                rows.push(vec!["<open>", "", ""]);
            }

            for mailbox in layer.mailboxes.iter() {
                rows.push(vec![
                    &format!("{}", mailbox.sort_order),
                    &mailbox.name,
                    &format!("{}", mailbox.unread_mails),
                ]);
            }

            rows
        };

        Table::new(
            rows,
            [
                Constraint::Length(2),
                Constraint::Fill(0),
                Constraint::Fill(0),
            ],
        )
    };

    StatefulWidget::render(
        list::List::new(layer.mailbox_owner.as_ref(), &layer.mailboxes),
        area,
        buf,
        &mut layer.state,
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
