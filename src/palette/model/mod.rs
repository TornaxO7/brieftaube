use crossterm::event::{Event, KeyCode};
use nucleo::Nucleo;
use ratatui::{style::Style, widgets::ListState};
use ratatui_textarea::TextArea;
use std::sync::Arc;

type EntryName = String;
type EntryDescription = String;
type EntryId = String;

pub type Callback = &'static dyn Fn(EntryId);

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: EntryId,
    /// The name which can be selected in the palette.
    pub name: EntryName,
    /// The description of the entry.
    pub description: EntryDescription,
}

pub struct Model {
    pub input: TextArea<'static>,
    pub nucleo: Nucleo<(EntryName, EntryDescription, EntryId)>,
    callback: Callback,

    pub list_state: ListState,
}

impl Model {
    pub fn new(entries: Vec<Entry>, callback: Callback) -> Self {
        let nucleo: Nucleo<(EntryName, EntryDescription, EntryId)> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 3);

        let inj = nucleo.injector();
        for e in entries.into_iter() {
            inj.push(
                (e.name, e.description, e.id),
                |&(ref name, ref description, ref id), row| {
                    row[0] = (*name).clone().into();
                    row[1] = (*description).clone().into();
                    row[2] = (*id).clone().into();
                },
            );
        }

        let input = {
            let mut input = TextArea::default();
            input.set_cursor_line_style(Style::new());
            input
        };

        Self {
            input,
            nucleo,
            callback,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Option<crate::Action> {
        match event {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Esc => {
                        return Some(crate::Action::Back);
                    }
                    KeyCode::Enter => {
                        let mut matches = self.nucleo.snapshot().matched_items(..);

                        if let Some(idx) = self.list_state.selected() {
                            let item = matches.nth(idx).unwrap();
                            let id = item.data.2.clone();

                            (self.callback)(id);
                        }

                        return Some(crate::Action::Back);
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
            }
            _ => {}
        }

        None
    }
}
