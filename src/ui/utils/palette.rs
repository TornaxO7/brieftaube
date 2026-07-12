use crate::ui::ScreenOverlayResult;
use crossterm::event::{KeyCode, KeyEvent};
use nucleo::Nucleo;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{
        Block, List, ListDirection, ListState, Padding, Paragraph, StatefulWidget, Widget, Wrap,
    },
};
use ratatui_textarea::TextArea;
use std::{marker::PhantomData, sync::Arc};

type EntryName = String;
type EntryDescription = String;
type EntryIdx = usize;

#[derive(Debug, Clone)]
pub struct Entry<E> {
    /// The value of the entry.
    pub value: E,
    /// The name which can be selected in the palette.
    pub name: String,
    /// The description of the entry.
    pub description: String,
}

pub struct State<E> {
    input: TextArea<'static>,
    nucleo: Nucleo<(EntryName, EntryDescription, EntryIdx)>,

    list_state: ListState,
    entries: Vec<E>,
}

impl<E: Clone> State<E> {
    pub fn new(entries: Vec<Entry<E>>) -> Self {
        let nucleo: Nucleo<(EntryName, EntryDescription, EntryIdx)> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 3);

        let mut new_entries = Vec::with_capacity(entries.len());
        let inj = nucleo.injector();
        for (idx, e) in entries.into_iter().enumerate() {
            new_entries.push(e.value);

            inj.push(
                (e.name, e.description, idx),
                |&(ref name, ref description, idx), row| {
                    row[0] = (*name).clone().into();
                    row[1] = (*description).clone().into();
                    row[2] = format!("{}", idx).into();
                },
            );
        }

        Self {
            input: TextArea::default(),
            nucleo,
            list_state: ListState::default().with_selected(Some(0)),
            entries: new_entries,
        }
    }

    pub fn handle_event<I>(&mut self, event: KeyEvent) -> Option<ScreenOverlayResult<E, I>> {
        match event.code {
            KeyCode::Esc => {
                self.reset();
                return Some(ScreenOverlayResult::Cancel);
            }
            KeyCode::Enter => {
                let result = {
                    let mut matches = self.nucleo.snapshot().matched_items(..);

                    if let Some(idx) = self.list_state.selected() {
                        let item = matches.nth(idx).unwrap();
                        let idx = item.data.2;

                        ScreenOverlayResult::Palette(self.entries[idx].clone())
                    } else {
                        ScreenOverlayResult::Cancel
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

pub struct Palette<E> {
    _phantom: PhantomData<E>,
}

impl<E> Palette<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E> StatefulWidget for Palette<E> {
    type State = State<E>;

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
