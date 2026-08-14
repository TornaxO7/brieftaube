use crate::palette::Model;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, List, ListDirection, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct Palette;

impl StatefulWidget for Palette {
    type State = Model;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let command_palette_block = Block::default().padding(Padding::symmetric(1, 1));
        let area = command_palette_block.inner(area);

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

        {
            let options_content: Vec<&str> = matches
                .iter()
                .map(|output| output.data.0.as_str())
                .collect();

            StatefulWidget::render(
                List::new(options_content)
                    .block(Block::bordered())
                    .highlight_style(Style::new().blue())
                    .direction(ListDirection::TopToBottom),
                options,
                buf,
                &mut state.list_state,
            );
        }
    }
}
