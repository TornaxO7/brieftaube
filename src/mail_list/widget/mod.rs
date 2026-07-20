mod render_data;

use super::State;
use crate::{
    backend::mails::types::{MailKeyword, ThreadMarker},
    utils::ui::{
        ScreenOverlay, ScreenState,
        input::Input,
        palette::Palette,
        symbol::{ATTACHMENT_SYMBOL, CHECKMARK, UNREAD_SYMBOL},
    },
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Style,
        palette::material::{BLUE, BLUE_GRAY, GRAY, GREEN, ORANGE},
    },
    text::Text,
    widgets::{Block, Cell, Clear, Paragraph, StatefulWidget, Table, Widget},
};
pub use render_data::RenderData;

pub const THREAD_BRANCH: &str = "├─";
pub const THREAD_LAST: &str = "╰─";

#[derive(Default)]
pub struct RootMails {}

impl StatefulWidget for RootMails {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(mut data) = state.get_render_data() {
            let [mails_area, preview_area] = area.layout(&Layout::horizontal([
                Constraint::Percentage(60),
                Constraint::Fill(1),
            ]));

            render_mail_list(mails_area, buf, &mut data);
            render_preview(preview_area, buf, &mut data);
        } else {
            // loading screen
        }

        render_overlay(area, buf, state);
    }
}

fn render_mail_list<'a>(area: Rect, buf: &mut Buffer, data: &mut RenderData<'a>) {
    const DATE_EXAMPLE: &str = "Mon, 15 May 2015, HH:MM:SS";

    let rows: Vec<ratatui::widgets::Row<'_>> = data
        .rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let subject = match row.thread_marker {
                ThreadMarker::Root => row.subject.clone(),
                ThreadMarker::Child => match data.rows.get(idx + 1) {
                    Some(next) => match next.thread_marker {
                        ThreadMarker::Root => format!("{} {}", THREAD_LAST, row.subject),
                        ThreadMarker::Child => format!("{} {}", THREAD_BRANCH, row.subject),
                    },
                    None => format!("{} {}", THREAD_LAST, row.subject),
                },
            };

            let has_attachment = if row.has_attachment {
                ATTACHMENT_SYMBOL
            } else {
                ""
            };

            let is_unread = if !row.keywords.contains(&MailKeyword::Seen) {
                UNREAD_SYMBOL
            } else {
                ""
            };

            let is_selected = if data.selected.contains(&row.id) {
                CHECKMARK
            } else {
                ""
            };

            let subject_style = if row.keywords.contains(&MailKeyword::Flagged) {
                Style::default().fg(ORANGE.c300)
            } else if !row.keywords.contains(&MailKeyword::Seen) {
                Style::default().fg(BLUE.c200)
            } else {
                Style::default()
            };

            ratatui::widgets::Row::new(vec![
                Cell::from(is_selected).style(Style::default().fg(GREEN.c400)),
                Cell::from(is_unread).style(Style::default().fg(BLUE.c400)),
                Cell::from(subject).style(subject_style),
                Cell::from(has_attachment).style(Style::default().fg(GRAY.c400)),
                Cell::from(row.from.clone()).style(Style::default().fg(BLUE_GRAY.c300)),
                Cell::from(row.received_at.clone()).style(Style::default().fg(GRAY.c400)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Min(DATE_EXAMPLE.len() as u16),
        ],
    )
    .header(
        ratatui::widgets::Row::new(["", "", "Subject", "", "From", "Received at"])
            .style(Style::default().underlined()),
    )
    .row_highlight_style(Style::default().white().bg(BLUE.c700))
    .block(Block::bordered());

    StatefulWidget::render(table, area, buf, data.table_state);
}

fn render_preview(area: Rect, buf: &mut Buffer, data: &mut RenderData) {
    if let Some(idx) = data.table_state.selected() {
        let mail = &data.rows[idx];

        let headers = vec![
            ("Received at:", mail.received_at.as_str()),
            ("From:", mail.from.as_str()),
            ("To:", mail.to.as_str()),
            ("Subject:", mail.subject.as_str()),
            ("Cc:", mail.cc.as_str()),
        ];

        let [header_area, preview_area] = Layout::vertical([
            Constraint::Length(headers.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .areas(area);

        render_headers(header_area, buf, &headers);

        Widget::render(
            Paragraph::new(mail.preview.as_str()).block(Block::bordered()),
            preview_area,
            buf,
        );
    } else {
        Widget::render(
            Paragraph::new("loading...").block(Block::bordered()),
            area,
            buf,
        );
    }
}

fn render_headers(area: Rect, buf: &mut Buffer, headers: &[(&'static str, &str)]) {
    let table = {
        let rows: Vec<ratatui::widgets::Row<'_>> = headers
            .iter()
            .map(|(name, value)| {
                ratatui::widgets::Row::new([
                    Cell::new(Text::from(*name).right_aligned())
                        .style(Style::default().fg(BLUE_GRAY.c400)),
                    Cell::new(*value),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(
                headers
                    .iter()
                    .map(|(header, _)| header.len())
                    .max()
                    .unwrap_or(5) as u16,
            ),
            Constraint::Fill(1),
        ];

        Table::new(rows, widths).block(Block::bordered())
    };

    Widget::render(table, area, buf);
}

fn render_overlay(area: Rect, buf: &mut Buffer, state: &mut State) {
    if let Some(state) = state.overlay() {
        let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
        Widget::render(Clear, a, buf);
        match state {
            ScreenOverlay::Palette(state) => {
                StatefulWidget::render(Palette::new(), a, buf, state);
            }
            ScreenOverlay::Input(state) => StatefulWidget::render(Input::new(), a, buf, state),
        }
    }
}
