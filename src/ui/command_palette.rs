use crossterm::event::{KeyCode, KeyEvent};
use nucleo::Nucleo;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};
use ratatui_textarea::TextArea;
use std::sync::Arc;

type EntryName = String;
type EntryDescription = String;

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum HandleEventResult {
    Selected(EntryName),
    Cancel,
}

pub struct CommandPalette {
    input: TextArea<'static>,
    nucleo: Nucleo<(EntryName, EntryDescription)>,

    list_state: ListState,
}

impl std::fmt::Debug for CommandPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").field("input", &self.input).finish()
    }
}

impl CommandPalette {
    pub fn new(entries: Vec<Entry>) -> Self {
        let nucleo: Nucleo<(String, String)> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 2);

        let inj = nucleo.injector();
        for e in entries {
            inj.push(
                (e.name, e.description),
                |&(ref name, ref description), row| {
                    row[0] = (*name).clone().into();
                    row[1] = (*description).clone().into();
                },
            );
        }

        Self {
            input: TextArea::default(),
            nucleo,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<HandleEventResult> {
        match event.code {
            KeyCode::Esc => {
                self.reset();
                return Some(HandleEventResult::Cancel);
            }
            KeyCode::Enter => {
                let result = {
                    let mut matches = self.nucleo.snapshot().matched_items(..);

                    if let Some(idx) = self.list_state.selected() {
                        HandleEventResult::Selected(matches.nth(idx).unwrap().data.0.clone())
                    } else {
                        HandleEventResult::Cancel
                    }
                };

                self.reset();
                return Some(result);
            }
            KeyCode::Down => {
                self.list_state.select_next();
                return None;
            }
            KeyCode::Up => {
                self.list_state.select_previous();
                return None;
            }
            _ => {}
        }

        self.input.input(event);

        let search_term = self.input.lines().get(0).unwrap().as_str();
        self.nucleo.pattern.reparse(
            0,
            search_term,
            nucleo::pattern::CaseMatching::Smart,
            nucleo::pattern::Normalization::Smart,
            false,
        );

        None
    }

    pub fn reset(&mut self) {
        self.input.clear();

        // reset search
        self.nucleo.pattern.reparse(
            0,
            "",
            nucleo::pattern::CaseMatching::Smart,
            nucleo::pattern::Normalization::Smart,
            false,
        );
    }
}

impl Widget for &mut CommandPalette {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
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
        self.nucleo.tick(10);
        let snapshot = self.nucleo.snapshot();
        let matches: Vec<_> = snapshot.matched_items(..).collect();
        if !matches.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }

        // description
        if let Some(selected) = self.list_state.selected() {
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
            self.input.set_block(Block::bordered().title("Search"));
            self.input.render(search, buf);
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
                &mut self.list_state,
            );
        }
    }
}
