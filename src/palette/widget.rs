use crate::palette::Model;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{
        Style,
        palette::material::{BLUE, BLUE_GRAY, CYAN, INDIGO, LIGHT_BLUE},
    },
    text::{Line, Span},
    widgets::{Block, List, ListDirection, ListItem, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct Palette;

impl StatefulWidget for Palette {
    type State = Model;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [left, description] = area.layout(
            &Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)]),
        );

        let [search, options] = left.layout(
            &Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(0)]),
        );

        // refresh state
        state.nucleo.tick(10);
        let snapshot = state.nucleo.snapshot();
        let matches: Vec<_> = snapshot.matched_items(..).collect();
        if !matches.is_empty() && state.list_state.selected().is_none() {
            state.list_state.select(Some(0));
        }

        // description
        if let Some(selected) = state.list_state.selected() {
            if let Some(description_content) = matches.get(selected) {
                Widget::render(
                    Paragraph::new(description_content.data.1.as_str())
                        .wrap(Wrap { trim: true })
                        .block(Block::bordered()),
                    description,
                    buf,
                );
            }
        }

        // search field
        {
            state.input.set_block(Block::bordered().title("Search"));
            state.input.render(search, buf);
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
                                Style::new().bold(),
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

            StatefulWidget::render(
                List::new(options_content)
                    .block(Block::bordered())
                    .highlight_style(Style::new().bg(BLUE.c900))
                    .direction(ListDirection::TopToBottom),
                options,
                buf,
                &mut state.list_state,
            );
        }
    }
}
