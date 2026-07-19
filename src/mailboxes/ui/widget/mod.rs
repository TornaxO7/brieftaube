use crate::{
    mailboxes::{
        Layer,
        ui::{State, state::SelectionType},
    },
    utils::{
        MailboxId,
        ui::{ScreenOverlay, ScreenState, input::Input, palette::Palette},
    },
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Style,
        palette::material::{BLUE, ORANGE},
    },
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, StatefulWidget, Table, Widget,
    },
};
use std::collections::HashMap;

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

                let left_block = Block::new().borders(Borders::RIGHT);
                let left_inner = left_block.inner(left);
                left_block.render(left, buf);
                if let Some(parent_layer) = data.layers.get_parent_layer_mut() {
                    render_layer(left_inner, buf, &state.selected, parent_layer);
                }

                render_layer(
                    center,
                    buf,
                    &state.selected,
                    data.layers.get_current_layer_mut(),
                );

                let right_block = Block::new().borders(Borders::LEFT);
                let right_inner = right_block.inner(right);
                right_block.render(right, buf);
                if let Some(children_layer) = data.layers.get_children_layer_mut() {
                    render_layer(right_inner, buf, &state.selected, children_layer);
                }
            } else {
                render_loading_screen(area, buf);
            }
        }

        render_overlay(area, buf, state);
    }
}

fn render_layer(
    area: Rect,
    buf: &mut Buffer,
    selected: &HashMap<MailboxId, SelectionType>,
    layer: &mut Layer,
) {
    let [marker_area, layer_area] =
        Layout::horizontal([Constraint::Length(1), Constraint::Fill(0)]).areas(area);

    render_marker(marker_area, buf, selected, layer);
    render_mailboxes(layer_area, buf, layer);
}

fn render_marker(
    area: Rect,
    buf: &mut Buffer,
    selected: &HashMap<MailboxId, SelectionType>,
    layer: &Layer,
) {
    let mut items: Vec<ListItem> = {
        let header_line = ListItem::new("");
        let open_line = ListItem::new("");

        if layer.is_root_layer() {
            vec![header_line]
        } else {
            vec![header_line, open_line]
        }
    };

    for mailbox in layer.mailboxes.iter() {
        let style = match selected.get(&mailbox.id) {
            Some(SelectionType::Selected) => Style::default().bg(ORANGE.c800),
            Some(SelectionType::Cut) => Style::default().on_red(),
            None => Style::default(),
        };
        items.push(ListItem::new(" ").style(style));
    }

    Widget::render(List::new(items), area, buf)
}

fn render_mailboxes(area: Rect, buf: &mut Buffer, layer: &mut Layer) {
    let table = {
        let rows = {
            let mut rows = Vec::with_capacity(layer.mailboxes.capacity() + 1);

            if !layer.is_root_layer() {
                rows.push(Row::new(["", "<open>"]).style(Style::default().yellow()));
            }

            for mailbox in layer.mailboxes.iter() {
                rows.push(Row::new(vec![
                    Cell::from(format!("{}", mailbox.sort_order)),
                    Cell::from(mailbox.name.clone()),
                    Cell::from(format!("{}", mailbox.unread_mails)),
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
        .header(Row::new(["Sort order", "Name", "Unread"]).style(Style::default().underlined()))
        .row_highlight_style(Style::default().bg(BLUE.c700))
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
                let area = area.centered(Constraint::Percentage(30), Constraint::Length(3));
                Widget::render(Clear, area, buf);
                StatefulWidget::render(Input::new(), area, buf, state)
            }
        }
    }
}
