mod render_data;

use crate::mail_viewer::{
    model::ScrollAction,
    widget::render_data::{RenderData, ViewerState},
};
use pulldown_cmark_mdcat::ratatui::{RenderOptions, Renderer};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::{
        Style,
        palette::material::{BLACK, BLUE, PINK, YELLOW},
    },
    text::Text,
    widgets::{
        Block, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Table, Tabs, Widget,
    },
};

pub struct MailViewer;

impl StatefulWidget for MailViewer {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        let mut data = RenderData::new(model);

        let [main_panel, tab_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

        render_tabs(tab_area, buf, &data);
        render_viewer(main_panel, buf, &mut data);
    }
}

fn render_tabs(area: Rect, buf: &mut Buffer, data: &RenderData) {
    let idx = match data.viewer_state {
        ViewerState::Metadata(_) => 0,
        ViewerState::Text { .. } => 1,
        ViewerState::Markdown { .. } => 2,
        ViewerState::Attachments(_) => 3,
    };

    Widget::render(
        Tabs::new(["Metadata", "Text", "Markdown (HTML)", "Attachments"])
            .block(Block::bordered().title("Tabs"))
            .highlight_style(Style::new().fg(YELLOW.c500))
            .select(Some(idx)),
        area,
        buf,
    );
}

/// Rendering implementations
fn render_viewer(area: Rect, buf: &mut Buffer, data: &mut RenderData) {
    match &mut data.viewer_state {
        ViewerState::Metadata(_state) => {
            const HEADERS: [&str; 6] = [
                "Received at:",
                "From:",
                "To:",
                "Subject:",
                "Cc:",
                "Keywords:",
            ];
            const LONGEST_HEADER_NAME: usize = longest_header_name(&HEADERS);

            let rows: Vec<Row> = [
                (HEADERS[0], &data.mail.received_at),
                (HEADERS[1], &data.mail.from),
                (HEADERS[2], &data.mail.to),
                (HEADERS[3], &data.mail.subject),
                (HEADERS[4], &data.mail.cc),
                (HEADERS[5], &data.mail.keywords),
            ]
            .iter()
            .map(|(header, value)| {
                let header = Text::raw(*header)
                    .alignment(HorizontalAlignment::Right)
                    .style(Style::new().fg(BLUE.c500));

                let value = Text::raw(*value);

                Row::new([header, value])
            })
            .collect();

            let widths = [
                Constraint::Length(LONGEST_HEADER_NAME as u16),
                Constraint::Fill(1),
            ];

            Widget::render(
                Table::new(rows, widths).block(Block::bordered().title("Metadata")),
                area,
                buf,
            );
        }
        ViewerState::Markdown {
            vertical,
            horizontal,
        } => {
            let Some(html) = data.mail.html_body.clone() else {
                Widget::render(
                    Paragraph::new("Fetching html body...").block(Block::bordered()),
                    area,
                    buf,
                );

                return;
            };
            let markdown = html_to_markdown_rs::convert(&html.0, None).unwrap();
            let content = markdown.content.unwrap();

            let renderer = Renderer::new(RenderOptions::default().width(area.width));
            let text = renderer.text_from_str(&content).unwrap();

            let (content_area, vertical_scrollbar_area, horizontal_scrollbar_area) =
                adjust_scrollbars(&text, area, vertical, horizontal, data.scroll_action);

            Widget::render(
                Paragraph::new(text).block(Block::bordered()).scroll((
                    vertical.get_position() as u16,
                    horizontal.get_position() as u16,
                )),
                content_area,
                buf,
            );

            if let Some(area) = vertical_scrollbar_area {
                StatefulWidget::render(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight),
                    area,
                    buf,
                    vertical,
                );
            }

            if let Some(area) = horizontal_scrollbar_area {
                StatefulWidget::render(
                    Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
                    area,
                    buf,
                    horizontal,
                );
            }
        }
        ViewerState::Text {
            vertical,
            horizontal,
        } => {
            let Some(content) = data.mail.text_body.as_ref() else {
                Widget::render(
                    Paragraph::new("Fetching text body part of mail...").block(Block::bordered()),
                    area,
                    buf,
                );
                return;
            };

            let text = Text::from(content.0.clone());

            let (content_area, vertical_scrollbar_area, horizontal_scrollbar_area) =
                adjust_scrollbars(&text, area, vertical, horizontal, data.scroll_action);

            Widget::render(
                Paragraph::new(text).block(Block::bordered()).scroll((
                    vertical.get_position() as u16,
                    horizontal.get_position() as u16,
                )),
                content_area,
                buf,
            );

            if let Some(area) = vertical_scrollbar_area {
                StatefulWidget::render(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight),
                    area,
                    buf,
                    vertical,
                );
            }

            if let Some(area) = horizontal_scrollbar_area {
                StatefulWidget::render(
                    Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
                    area,
                    buf,
                    horizontal,
                );
            }
        }

        ViewerState::Attachments(state) => {
            let Some(attachments) = &data.mail.attachments else {
                Widget::render(
                    Paragraph::new("Fetching attachments...").block(Block::bordered()),
                    area,
                    buf,
                );

                return;
            };

            if let Some(scroll) = data.scroll_action.take() {
                match scroll {
                    ScrollAction::ScrollUp(amount) => {
                        state.scroll_up_by(amount as u16);
                    }
                    ScrollAction::ScrollDown(amount) => {
                        state.scroll_down_by(amount as u16);
                    }
                    _ => {}
                }
            }

            let rows: Vec<Row<'_>> = attachments
                .iter()
                .map(|a| {
                    Row::new([
                        Text::raw(&a.size).style(Style::new().fg(YELLOW.c500)),
                        Text::raw(&a.name),
                        Text::raw(&a.content_type).style(Style::new().fg(PINK.c500)),
                    ])
                })
                .collect();

            let widths = [Constraint::Max(6), Constraint::Fill(1), Constraint::Fill(1)];

            StatefulWidget::render(
                Table::new(rows, widths)
                    .row_highlight_style(Style::new().fg(BLACK).bg(BLUE.c500))
                    .block(Block::bordered().title("Attachments")),
                area,
                buf,
                state,
            );
        }
    }
}

fn adjust_scrollbars(
    text: &Text,
    area: Rect,
    vertical: &mut ScrollbarState,
    horizontal: &mut ScrollbarState,
    queue: &mut Option<ScrollAction>,
) -> (Rect, Option<Rect>, Option<Rect>) {
    let amount_unseen_lines = text.height().saturating_sub(area.height as usize);
    let amount_unseen_columns = text.width().saturating_sub(area.width as usize);

    if let Some(action) = queue.take() {
        match action {
            ScrollAction::ScrollUp(amount) => {
                let pos = vertical.get_position();
                *vertical = vertical.position(pos.saturating_sub(amount));
            }
            ScrollAction::ScrollDown(amount) => {
                let pos = vertical.get_position();
                *vertical = vertical.position((pos + amount).min(amount_unseen_lines));
            }
            ScrollAction::ScrollHalfPageDown => {
                let prev_pos = vertical.get_position();
                let new_pos = prev_pos + area.height as usize / 2;
                *vertical = vertical.position(new_pos.min(amount_unseen_lines));
            }
            ScrollAction::ScrollHalfPageUp => {
                let prev_pos = vertical.get_position();
                *vertical = vertical.position(prev_pos.saturating_sub(area.height as usize / 2));
            }
            ScrollAction::SetTop => vertical.first(),
            ScrollAction::SetBottom => vertical.last(),
            ScrollAction::ScrollHalfPageRight => {
                let prev_pos = horizontal.get_position();
                let new_pos = prev_pos + area.width as usize / 2;
                *horizontal = horizontal.position(new_pos.min(amount_unseen_columns));
            }
            ScrollAction::ScrollRight(amount) => {
                let prev_pos = horizontal.get_position();
                let new_pos = prev_pos + amount;
                *horizontal = horizontal.position(new_pos.min(amount_unseen_columns));
            }
            ScrollAction::ScrollHalfPageLeft => {
                let prev_pos = horizontal.get_position();
                *horizontal = horizontal.position(prev_pos.saturating_sub(area.width as usize / 2));
            }
            ScrollAction::ScrollLeft(amount) => {
                let prev_pos = horizontal.get_position();
                *horizontal = horizontal.position(prev_pos.saturating_sub(amount));
            }
        }
    }

    // restrict height
    *vertical = vertical.content_length(amount_unseen_lines);
    let (rest, vertical_scrollbar_area) = {
        let scrollbar_is_visible = amount_unseen_lines > 0;
        if scrollbar_is_visible {
            let [rest, scrollbar_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

            (rest, Some(scrollbar_area))
        } else {
            (area, None)
        }
    };

    // restrict width
    *horizontal = horizontal.content_length(amount_unseen_columns);
    let (mail_content_area, horizontal_scrollbar_area) = {
        let scrollbar_is_visible = amount_unseen_columns > 0;
        if scrollbar_is_visible {
            let [mail_content_area, scrollbar_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(rest);

            (mail_content_area, Some(scrollbar_area))
        } else {
            (rest, None)
        }
    };

    (
        mail_content_area,
        vertical_scrollbar_area,
        horizontal_scrollbar_area,
    )
}

// I wish `const` in rust would be more powerful :(
const fn longest_header_name(headers: &[&str]) -> usize {
    let mut max = 0;
    let mut i = 0;
    while i < headers.len() {
        if headers[i].len() > max {
            max = headers[i].len();
        }
        i += 1;
    }
    max
}
