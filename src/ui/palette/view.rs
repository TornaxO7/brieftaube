use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Clear, List, ListDirection, ListItem, Paragraph, Wrap},
};

use crate::{THEME, utils::IntoColor};

pub fn view(state: &mut super::State, frame: &mut Frame, area: Rect) {
    let theme = THEME.get().unwrap();
    let scheme = &theme.schemes.dark;

    let centered_area = area.centered(Constraint::Percentage(80), Constraint::Percentage(80));
    frame.render_widget(Clear, centered_area);

    let [left_area, description_area] =
        Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)])
            .areas(centered_area);

    let [search_area, options_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(0)]).areas(left_area);

    // get snapshot
    let snapshot = state.nucleo.snapshot();
    tracing::debug!("{}, {}", snapshot.matched_item_count(), options_area.height);
    let max_amount = snapshot
        .matched_item_count()
        .min(options_area.height as u32);
    let matches: Vec<_> = snapshot.matched_items(..max_amount).collect();
    if !matches.is_empty() && state.list_state.selected().is_none() {
        state.list_state.select(Some(0));
    }

    // search field
    {
        state.input.set_block(
            Block::bordered()
                .title("Search")
                .border_style(scheme.outline.into_color()),
        );
        frame.render_widget(&state.input, search_area);
    }

    // description
    {
        let description_block =
            Block::bordered().border_style(Style::new().fg(scheme.outline.into_color()));
        let inner_description_area = description_block.inner(description_area);

        frame.render_widget(description_block, description_area);

        if let Some(selected) = state.list_state.selected() {
            if let Some(description_content) = matches.get(selected) {
                frame.render_widget(
                    Paragraph::new(description_content.data.1.as_str())
                        .style(Style::new().fg(scheme.on_surface.into_color()))
                        .wrap(Wrap { trim: true }),
                    inner_description_area,
                );
            }
        }
    }

    // options
    {
        let search_term = state.get_search_term();

        let options_content: Vec<ListItem> = matches
            .iter()
            .map(|output| {
                let value = output.data.0.as_str();

                let spans: Vec<Span> = {
                    let mut spans = Vec::new();

                    let mut start = 0;
                    for (match_idx, _) in value.match_indices(search_term) {
                        if match_idx > start {
                            spans.push(Span::raw(&value[start..match_idx]));
                        }

                        spans.push(Span::styled(
                            &value[match_idx..(match_idx + search_term.len())],
                            Style::new()
                                .bold()
                                .fg(scheme.on_primary_container.into_color()),
                        ));

                        start = match_idx + search_term.len();
                    }

                    if start < value.len() {
                        spans.push(Span::raw(&value[start..]));
                    }

                    spans
                };

                ListItem::new(Line::from(spans))
            })
            .collect();

        frame.render_stateful_widget(
            List::new(options_content)
                .block(Block::bordered().border_style(scheme.outline.into_color()))
                .highlight_style(
                    Style::new()
                        .fg(scheme.on_primary.into_color())
                        .bg(scheme.primary.into_color()),
                )
                .direction(ListDirection::TopToBottom),
            options_area,
            &mut state.list_state,
        );
    }
}
