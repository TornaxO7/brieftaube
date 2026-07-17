use crate::{
    mailboxes::{Layer, ui::State},
    utils::ui::{ScreenOverlay, ScreenState, input::Input, palette::Palette},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph, Row, StatefulWidget, Table, Widget},
};

const DARK_TURQUOISE: Color = Color::from_u32(0x005eff);

#[derive(Default)]
pub struct Mailboxes {}

impl StatefulWidget for Mailboxes {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        {
            let mut guard = state.backend.data.lock().unwrap();

            if let Some(data) = guard.as_mut() {
                let [left, center, right] = Layout::horizontal([
                    Constraint::Fill(0),
                    Constraint::Fill(0),
                    Constraint::Fill(0),
                ])
                .areas(area);

                if let Some(parent_layer) = data.layers.get_parent_layer_mut() {
                    render_layer(left, buf, parent_layer);
                }

                render_layer(center, buf, data.layers.get_current_layer_mut());

                if let Some(children_layer) = data.layers.get_children_layer_mut() {
                    render_layer(right, buf, children_layer);
                }
            } else {
                render_loading_screen(area, buf);
            }
        }

        render_overlay(area, buf, state);
    }
}

fn render_layer(area: Rect, buf: &mut Buffer, layer: &mut Layer) {
    let table = {
        let rows = {
            let mut rows = Vec::with_capacity(layer.mailboxes.capacity() + 1);

            if !layer.is_root_layer() {
                rows.push(Row::new(["", "<open>"]).style(Style::default().yellow()));
            }

            for mailbox in layer.mailboxes.iter() {
                rows.push(Row::new(vec![
                    format!("{}", mailbox.sort_order),
                    mailbox.name.clone(),
                    format!("{}", mailbox.unread_mails),
                ]));
            }

            rows
        };

        Table::new(
            rows,
            [
                Constraint::Min(3),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ],
        )
        .header(Row::new(["Sort order", "Name", "Unread"]))
        .row_highlight_style(Style::default().bg(DARK_TURQUOISE))
    };

    StatefulWidget::render(table, area, buf, &mut layer.state)
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

fn render_overlay(area: Rect, buf: &mut Buffer, state: &mut State) {
    if let Some(state) = state.overlay() {
        match state {
            ScreenOverlay::Palette(state) => {
                let area = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
                Widget::render(Clear, area, buf);
                StatefulWidget::render(Palette::new(), area, buf, state);
            }
            ScreenOverlay::Input(state) => {
                let area = area.centered(Constraint::Percentage(20), Constraint::Percentage(15));
                Widget::render(Clear, area, buf);
                StatefulWidget::render(Input::new(), area, buf, state)
            }
        }
    }
}
