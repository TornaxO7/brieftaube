pub mod types;

use crate::{
    mailboxes::state::SelectionType,
    utils::ui::{
        ScreenOverlay, ScreenState,
        input::Input,
        palette::Palette,
        symbol::{CHECKMARK, SCISSORS},
    },
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Style,
        palette::material::{BLACK, BLUE, GRAY, GREEN, RED, YELLOW},
    },
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, StatefulWidget, Table, Widget},
};
use types::*;

#[derive(Default)]
pub struct Mailboxes {}

impl StatefulWidget for Mailboxes {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        {
            if state.is_waiting_for_data() {
                render_loading_screen(area, buf);
            } else {
                let [left, center, right] = Layout::horizontal([
                    Constraint::Fill(0),
                    Constraint::Fill(0),
                    Constraint::Fill(0),
                ])
                .areas(area);

                render_left_column(left, buf, state);
                render_center_column(center, buf, state);
                render_right_column(right, buf, state);
            }

            render_overlay(area, buf, state);
        }
    }
}

fn render_column<'a>(area: Rect, buf: &mut Buffer, data: ColumnData<'a>) {
    let table = {
        let rows: Vec<Row> = data
            .mailboxes
            .into_iter()
            .map(|mailbox| match mailbox {
                MailboxDisplay::This => Row::new([
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new("open")
                        .column_span(2)
                        .style(Style::new().fg(YELLOW.c700).italic()),
                ]),
                MailboxDisplay::Entry {
                    selection_type,
                    sort_order,
                    name,
                    unread_mails,
                } => {
                    let selection = match selection_type {
                        Some(SelectionType::Selected) => {
                            Cell::from(CHECKMARK).style(Style::new().fg(GREEN.c400))
                        }
                        Some(SelectionType::Cut) => {
                            Cell::from(SCISSORS).style(Style::new().fg(RED.c400))
                        }
                        None => Cell::from(""),
                    };

                    let sort_order = format!("{}", sort_order);
                    let unread_mails = {
                        let s = format!("{}", unread_mails);

                        let style = if unread_mails == 0 {
                            Style::new().fg(GRAY.c500)
                        } else {
                            Style::new().fg(GREEN.c500)
                        };

                        Cell::from(s).style(style)
                    };

                    Row::new([
                        selection,
                        Cell::from(sort_order).style(Style::new().fg(GRAY.c400)),
                        Cell::from(name).style(Style::new().fg(GRAY.c200)),
                        unread_mails,
                    ])
                }
            })
            .collect();

        Table::new(rows, ColumnData::widths())
            .header(ColumnData::header())
            .row_highlight_style(Style::default().white().bg(BLUE.c600).fg(BLACK))
    };

    if let Some(state) = data.state {
        StatefulWidget::render(table, area, buf, state);
    } else {
        Widget::render(table, area, buf);
    }
}

fn render_left_column(area: Rect, buf: &mut Buffer, state: &mut super::State) {
    let left_block = Block::new().borders(Borders::RIGHT);
    let left_inner = left_block.inner(area);
    left_block.render(area, buf);

    if let Some(parent_column) = state.get_parent_column() {
        render_column(left_inner, buf, parent_column);
    }
}

fn render_center_column(area: Rect, buf: &mut Buffer, state: &mut super::State) {
    render_column(area, buf, state.get_center_column());
}

fn render_right_column(area: Rect, buf: &mut Buffer, state: &mut super::State) {
    let right_block = Block::new().borders(Borders::LEFT);
    let right_inner = right_block.inner(area);
    right_block.render(area, buf);

    if let Some(children_column) = state.get_right_column() {
        render_column(right_inner, buf, children_column);
    }
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
