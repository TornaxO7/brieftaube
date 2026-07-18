use super::State;
use crate::{
    root_mails::backend::MailRenderable,
    utils::ui::{DARK_TURQUOISE, ScreenOverlay, ScreenState, input::Input, palette::Palette},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Cell, Clear, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};

#[derive(Default)]
pub struct RootMails {}

impl StatefulWidget for RootMails {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        {
            let mut guard = state.backend.data.lock().unwrap();

            if let Some(data) = guard.as_mut() {
                let mails = &data.mails_renderable;

                let [mails_area, preview_area] = area.layout(&Layout::horizontal([
                    Constraint::Percentage(60),
                    Constraint::Percentage(40),
                ]));

                render_mail_list(mails_area, buf, &mails, &mut data.table_state);

                let selected_mail = data.table_state.selected().and_then(|idx| mails.get(idx));
                render_preview(preview_area, buf, selected_mail);
            } else {
            }
        }

        render_overlay(area, buf, state);
    }
}

fn render_mail_list(
    area: Rect,
    buf: &mut Buffer,
    mails: &Vec<MailRenderable>,
    state: &mut TableState,
) {
    const DATE_EXAMPLE: &str = "15 May 2015, HH:MM:SS";

    let area = Block::default().inner(area);

    let table = {
        let rows: Vec<Row<'_>> = mails
            .iter()
            .map(|mail| {
                Row::new(vec![
                    Cell::from(mail.subject.as_str()),
                    Cell::from(mail.from.as_str()),
                    Cell::from(mail.received_at.as_str()),
                ])
            })
            .collect();

        Table::new(
            rows,
            [
                Constraint::Min(DATE_EXAMPLE.len() as u16),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
        .header(Row::new(["Subject", "From", "Received at"]).style(Style::default().underlined()))
        .row_highlight_style(Style::default().bg(DARK_TURQUOISE))
        .block(Block::bordered())
    };

    StatefulWidget::render(table, area, buf, state)
}

fn render_preview(area: Rect, buf: &mut Buffer, mail: Option<&MailRenderable>) {
    if let Some(mail) = mail {
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
        let rows: Vec<Row<'_>> = headers
            .iter()
            .map(|(name, value)| Row::new([Cell::new(*name), Cell::new(*value)]))
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
