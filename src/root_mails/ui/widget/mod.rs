mod list;

use super::State;
use crate::{
    root_mails::backend::Data,
    utils::ui::{DARK_TURQUOISE, ScreenOverlay, ScreenState, input::Input, palette::Palette},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Cell, Clear, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};

const MAIL_LIST_PANEL_TITLE: &str = "Mails";
const PREVIEW_PANEL_TITLE: &str = "Mail content";

#[derive(Default)]
pub struct RootMails {}

impl StatefulWidget for RootMails {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        {
            let mut guard = state.backend.data.lock().unwrap();

            if let Some(data) = guard.as_mut() {
                let [mails_area, preview_area] = area.layout(&Layout::horizontal([
                    Constraint::Percentage(60),
                    Constraint::Percentage(40),
                ]));

                render_mail_list(mails_area, buf, data);
                render_preview(preview_area, buf, &data);
            } else {
            }
        }

        render_overlay(area, buf, state);
    }
}

fn render_mail_list(area: Rect, buf: &mut Buffer, data: &mut Data) {
    const DATE_EXAMPLE: &str = "15 May 2015, HH:MM:SS";

    let table = {
        let rows: Vec<Row<'_>> = data
            .mails
            .iter()
            .map(|mail| {
                let date = mail.received_at.format("%e %b %Y, %H:%M:%S").to_string();
                let from = mail.from.iter().fold(String::new(), |acc, addr| {
                    format!("{acc}, {}", addr.to_string())
                });
                let subject = mail.subject.as_str();

                Row::new(vec![
                    Cell::from(date),
                    Cell::from(from),
                    Cell::from(subject),
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
        .header(Row::new(["Received at", "From", "Subject"]).style(Style::default().underlined()))
        .row_highlight_style(Style::default().bg(DARK_TURQUOISE))
    };

    StatefulWidget::render(table, area, buf, &mut data.table_state)
}

fn render_preview(area: Rect, buf: &mut Buffer, data: &Data) {
    if let Some(idx) = data.table_state.selected() {
        let mail = &data.mails[idx];
        Widget::render(Paragraph::new(mail.preview.as_str()), area, buf);
    } else {
        Widget::render(Paragraph::new("loading..."), area, buf);
    }
}

fn render_headers(area: Rect, buf: &mut Buffer, headers: &[(&'static str, &str)]) {
    todo!()
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
