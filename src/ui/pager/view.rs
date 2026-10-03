use crate::{
    ui::{
        pager::{MailHeaders, Mode, SelectedBodyType, SelectedTab, body::BodyReader},
        statusbar::StatusbarWidget,
    },
    utils::IntoColor,
};
use material_theme_loader::Scheme;
use ratatui::{
    Frame,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Style,
    text::Text,
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, Tabs,
    },
};
use throbber_widgets_tui::Throbber;

pub fn view(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let [tabs_area, content_area, statusbar_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    render_tab_widgets(scheme, state, frame, tabs_area);

    let msgs = match state.selected_tab {
        SelectedTab::Mail => render_mail_tab(scheme, state, frame, content_area),
        SelectedTab::Attachments => render_attachments_tab(scheme, state, frame, content_area),
    };

    render_statusbar(scheme, state, frame, statusbar_area);

    msgs
}

fn render_tab_widgets(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    const TAB1: &str = "Content";

    let tab2 = {
        let prefix = "Attachments";

        match state.attachments.get() {
            Some(attachments_tab) => match &attachments_tab.attachments {
                Ok(attachments) => format!("{} ({})", prefix, attachments.len()),
                Err(_) => format!("{} (E)", prefix),
            },
            None => format!("{} (?)", prefix),
        }
    };

    let selected_idx = match state.selected_tab {
        SelectedTab::Mail => 0,
        SelectedTab::Attachments => 1,
    };

    frame.render_widget(
        Tabs::new([TAB1, tab2.as_str()])
            .select(selected_idx)
            .block(Block::new().borders(Borders::TOP))
            .style(Style::new().fg(scheme.outline.into_color()))
            .highlight_style(Style::new().fg(scheme.primary.into_color())),
        area,
    );
}

fn render_mail_tab(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let body_area = render_mail_headers(scheme, state, frame, area);
    render_mail_body(scheme, state, frame, body_area)
}

fn render_mail_headers(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Rect {
    match state.headers.get() {
        None => {
            const LABEL: &str = "Loading headers";

            let [headers_area, body_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

            let headers_area = headers_area.centered(
                Constraint::Length(LABEL.len() as u16 + 2),
                Constraint::Length(1),
            );

            frame.render_stateful_widget(
                Throbber::default()
                    .label(LABEL)
                    .style(Style::new().fg(scheme.primary.into_color())),
                headers_area,
                &mut state.throbber,
            );

            body_area
        }
        Some(headers) => match headers {
            Ok(headers) => {
                let rows: Vec<Row<'_>> = {
                    let mut rows = Vec::with_capacity(MailHeaders::MAX_AMOUNT_HEADERS);

                    let header_style = Style::new().fg(scheme.primary.into_color()).bold();
                    let value_style = Style::new().fg(scheme.secondary.into_color());

                    if let Some(from) = &headers.from {
                        rows.push(Row::new(vec![
                            Cell::from("From:").style(header_style),
                            Cell::from(from.to_string()).style(value_style),
                        ]))
                    }

                    if let Some(to) = &headers.to {
                        rows.push(Row::new(vec![
                            Cell::from("To:").style(header_style),
                            Cell::from(to.to_string()).style(value_style),
                        ]));
                    }

                    if let Some(cc) = &headers.cc {
                        rows.push(Row::new(vec![
                            Cell::from("Cc:").style(header_style),
                            Cell::from(cc.to_string()).style(value_style),
                        ]));
                    }

                    if let Some(subject) = &headers.subject {
                        rows.push(Row::new(vec![
                            Cell::from("Subject:").style(header_style),
                            Cell::from(subject.as_str()).style(value_style),
                        ]));
                    }

                    rows.push(Row::new(vec![
                        Cell::from("Received at:").style(header_style),
                        Cell::from(headers.received_at.as_str()).style(value_style),
                    ]));

                    rows
                };

                let [headers_area, body_area] =
                    Layout::vertical([Constraint::Length(rows.len() as u16), Constraint::Fill(1)])
                        .areas(area);

                let widths = [
                    Constraint::Length(MailHeaders::LONGEST_HEADER_LENGTH as u16),
                    Constraint::Fill(1),
                ];

                frame.render_widget(
                    Table::new(rows, widths).block(
                        Block::new()
                            .borders(Borders::TOP)
                            .border_style(Style::new().fg(scheme.outline.into_color())),
                    ),
                    headers_area,
                );

                body_area
            }
            Err(err) => {
                let err_msg = err.to_string();
                let amount_lines = err_msg.lines().count();

                let [headers_area, body_area] = Layout::vertical([
                    Constraint::Length(amount_lines as u16),
                    Constraint::Fill(0),
                ])
                .areas(area);

                frame.render_widget(
                    Paragraph::new(err_msg).style(Style::new().fg(scheme.error.into_color())),
                    headers_area,
                );

                body_area
            }
        },
    }
}

fn render_mail_body(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let mut block = Block::new()
        .title_style(Style::new().fg(scheme.tertiary.into_color()))
        .border_style(Style::new().fg(scheme.outline.into_color()))
        .borders(Borders::TOP)
        .title_alignment(HorizontalAlignment::Right);
    let inner_area = block.inner(area);

    match state.selected_body_type {
        SelectedBodyType::Text => {
            block = block.title(" Type: Text");
            frame.render_widget(block, area);

            render_text_body(scheme, state, frame, inner_area)
        }
        SelectedBodyType::Html => {
            block = block.title(" Type: Markdown (Html)");
            frame.render_widget(block, area);

            render_html_body(scheme, state, frame, inner_area)
        }
    }
}

fn render_text_body(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let text_body = match state.text_body.get_mut() {
        Some(text_body) => text_body,
        None => {
            const LABEL: &str = "Loading text body";

            let area = area.centered(
                Constraint::Length(LABEL.len() as u16 + 2),
                Constraint::Length(1),
            );

            frame.render_stateful_widget(
                Throbber::default()
                    .label(LABEL)
                    .style(Style::new().fg(scheme.primary.into_color())),
                area,
                &mut state.throbber,
            );

            return vec![];
        }
    };

    let text_body = match text_body.as_mut() {
        Ok(text_body) => text_body.as_mut(),
        Err(err) => {
            let msg = format!("Couldn't get `text/body` of mail:\n{}", err.to_string());
            render_msg_centered(msg, Style::new().fg(scheme.error.into_color()), frame, area);
            return vec![];
        }
    };

    match text_body.and_then(|text_body| text_body.scrollable_body()) {
        Some(body) => {
            let [body_area, scrollbar_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

            body.set_content_height(body_area.as_size());

            frame.render_widget(
                body.default_paragraph()
                    .style(Style::new().fg(scheme.primary.into_color()))
                    .scroll((body.scroll_offset as u16, 0)),
                body_area,
            );

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                scrollbar_area,
                &mut ScrollbarState::new(body.content_height).position(body.scroll_offset),
            );

            vec![]
        }
        None => {
            const MSG: &str = "Mail doesn't have `text/body`.";
            render_msg_centered(
                MSG.to_string(),
                Style::new().fg(scheme.primary.into_color()),
                frame,
                area,
            );

            vec![]
        }
    }
}

fn render_html_body(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let html_body = match state.html_body.get_mut() {
        Some(html_body) => html_body,
        None => {
            // TODO: Maybe merge it with the loading screen of text-body? Like a generic one
            const LABEL: &str = "Loading html body";

            let area = area.centered(
                Constraint::Length(LABEL.len() as u16 + 2),
                Constraint::Length(1),
            );

            frame.render_stateful_widget(
                Throbber::default()
                    .label(LABEL)
                    .style(Style::new().fg(scheme.primary.into_color())),
                area,
                &mut state.throbber,
            );

            return vec![];
        }
    };

    let html_body = match html_body {
        Ok(html_body) => html_body.as_mut(),
        Err(err) => {
            let msg = format!("Couldn't get `html/body` of mail:\n{}", err.to_string());
            render_msg_centered(msg, Style::new().fg(scheme.error.into_color()), frame, area);
            return vec![];
        }
    };

    match html_body.and_then(|html_body| html_body.scrollable_body()) {
        Some(body) => {
            let [body_area, scrollbar_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

            body.set_content_height(body_area.as_size());

            frame.render_widget(
                body.default_paragraph()
                    .style(Style::new().fg(scheme.primary.into_color()))
                    .scroll((body.scroll_offset as u16, 0)),
                body_area,
            );

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                scrollbar_area,
                &mut ScrollbarState::new(body.content_height).position(body.scroll_offset),
            );

            vec![]
        }
        None => {
            const MSG: &str = "Mail doesn't have `html/body`.";

            render_msg_centered(
                MSG.to_string(),
                Style::new().fg(scheme.primary.into_color()),
                frame,
                area,
            );

            vec![]
        }
    }
}

fn render_attachments_tab(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) -> Vec<crate::ui::Message> {
    let attachments_tab = match state.attachments.get_mut() {
        Some(attachments) => attachments,
        None => {
            const LABEL: &str = "Loading attachments";
            const THROBBER_SYMBOL_WITH_SPACE: u16 = 2;

            let area = area.centered(
                Constraint::Length(LABEL.len() as u16 + THROBBER_SYMBOL_WITH_SPACE),
                Constraint::Length(1),
            );

            frame.render_stateful_widget(
                Throbber::default()
                    .label(LABEL)
                    .style(Style::new().fg(scheme.primary.into_color())),
                area,
                &mut state.throbber,
            );
            return vec![];
        }
    };

    match &attachments_tab.attachments {
        Ok(attachments) => {
            let mut longest_content_type_name = 0;
            let mut longest_size_name = 0;

            let rows: Vec<Row<'_>> = attachments
                .iter()
                .map(|attachment| {
                    let size_name = format_size(attachment.size);

                    longest_size_name = longest_size_name.max(size_name.len());

                    longest_content_type_name =
                        longest_content_type_name.max(attachment.content_type.len());

                    let size = Cell::from(Text::from(size_name.clone()).right_aligned())
                        .style(Style::new().fg(scheme.secondary.into_color()));
                    let name = Cell::from(attachment.name.as_str())
                        .style(Style::new().fg(scheme.primary.into_color()));
                    let content_type = Cell::from(attachment.content_type.as_str())
                        .style(Style::new().fg(scheme.tertiary.into_color()));

                    Row::new([size, name, content_type])
                })
                .collect();

            let widths = [
                Constraint::Length(longest_size_name as u16),
                Constraint::Fill(1),
                Constraint::Length(longest_content_type_name as u16),
            ];

            frame.render_stateful_widget(
                Table::new(rows, widths)
                    .row_highlight_style(
                        Style::new()
                            .fg(scheme.on_primary_container.into_color())
                            .bg(scheme.primary_container.into_color()),
                    )
                    .column_spacing(2)
                    .block(
                        Block::new()
                            .borders(Borders::TOP)
                            .style(Style::new().fg(scheme.outline.into_color())),
                    ),
                area,
                &mut attachments_tab.state,
            );
        }
        Err(err) => {
            let msg = format!("Couldn't retrieve attachments:\n{}", err.to_string());
            render_msg_centered(msg, Style::new().fg(scheme.error.into_color()), frame, area);
        }
    };

    vec![]
}

fn format_size(size: usize) -> String {
    const UNITS: [(usize, &str); 3] = [(1_000_000_000, "GB"), (1_000_000, "MB"), (1_000, "KB")];

    for (unit, suffix) in UNITS {
        if size >= unit {
            let tenths = (size * 10 + unit / 2) / unit;
            if tenths >= 10 {
                return format!("{},{}{suffix}", tenths / 10, tenths % 10);
            }
        }
    }

    format!("{size} B")
}

fn render_msg_centered(msg: String, style: Style, frame: &mut Frame, area: Rect) {
    let area = area.centered(
        Constraint::Length(msg.lines().map(|line| line.len()).max().unwrap_or(0) as u16),
        Constraint::Length(msg.lines().count() as u16),
    );

    frame.render_widget(Paragraph::new(msg).style(style), area);
}

fn render_statusbar(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let layer_name = format!(
        "Pager({})",
        match state.mode {
            Mode::Reader => "Reader",
            Mode::Composer => "Composer",
        }
    );

    frame.render_stateful_widget(
        StatusbarWidget::new(scheme).layer_name(layer_name.as_str()),
        area,
        &mut state.statusbar,
    );
}
