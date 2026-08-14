use crate::utils::layer::{LayerCore, LayerOverlay};
use crossterm::event::{Event, KeyCode};
use nucleo::Nucleo;
use ratatui::{style::Style, widgets::ListState};
use ratatui_textarea::TextArea;
use std::sync::Arc;

type EntryValue = String;
type EntryDescription = String;

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    /// The name which can be selected in the palette.
    pub name: EntryValue,
    /// The description of the entry.
    pub description: EntryDescription,
}

pub struct Model {
    pub input: TextArea<'static>,
    pub nucleo: Nucleo<(EntryValue, EntryDescription)>,

    pub list_state: ListState,

    selected_entry: Option<EntryValue>,
}

impl Model {
    pub fn new(entries: Vec<PaletteEntry>) -> Self {
        let nucleo: Nucleo<(EntryValue, EntryDescription)> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 3);

        let inj = nucleo.injector();
        for e in entries.into_iter() {
            inj.push(
                (e.name, e.description),
                |&(ref name, ref description), row| {
                    row[0] = (*name).clone().into();
                    row[1] = (*description).clone().into();
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
            selected_entry: None,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn get_search_term(&self) -> &str {
        self.input.lines()[0].as_str()
    }
}

impl LayerCore for Model {
    fn handle_event(
        &mut self,
        event: Event,
        _: &mut crate::statusbar::Model,
    ) -> Option<crate::Action> {
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

                            let value = item.data.0.clone();
                            self.selected_entry = Some(value);
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

impl LayerOverlay for Model {
    fn into_message(self) -> Option<String> {
        self.selected_entry
    }
}
