mod view;

use crate::ui::Layer;
use crossterm::event::{Event, KeyCode};
use nucleo::Nucleo;
use ratatui::{style::Style, widgets::ListState};
use ratatui_textarea::TextArea;
use std::sync::Arc;

pub use view::view;

type EntryValue = String;
type EntryDescription = String;

pub enum Message {
    Restart {
        entries: Vec<PaletteEntry>,
        map: fn(String) -> super::Message,
    },
    Event(Event),
}

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    /// The name which can be selected in the palette.
    pub name: EntryValue,
    /// The description of the entry.
    pub description: EntryDescription,
}

pub struct State {
    pub input: TextArea<'static>,
    pub nucleo: Nucleo<(EntryValue, EntryDescription)>,

    pub list_state: ListState,

    pub map: fn(String) -> super::Message,
}

impl State {
    pub fn new() -> Self {
        let nucleo: Nucleo<(EntryValue, EntryDescription)> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 3);

        let input = {
            let mut input = TextArea::default();
            input.set_cursor_line_style(Style::new());
            input
        };

        let map = |_| {
            unreachable!();
        };

        Self {
            input,
            nucleo,
            map,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn get_search_term(&self) -> &str {
        self.input.lines()[0].as_str()
    }
}

impl Layer<Message, super::Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        match msg {
            Message::Restart { entries, map } => self.handle_restart(entries, map),
            Message::Event(event) => self.handle_event(event),
        }
    }
}

impl State {
    fn handle_restart(
        &mut self,
        entries: Vec<PaletteEntry>,
        map: fn(String) -> super::Message,
    ) -> Vec<super::Message> {
        self.nucleo.restart(true);
        self.input.clear();
        self.map = map;
        self.list_state.select(Some(0));

        let inj = self.nucleo.injector();
        for e in entries.into_iter() {
            inj.push(
                (e.name, e.description),
                |&(ref name, ref description), row| {
                    row[0] = (*name).clone().into();
                    row[1] = (*description).clone().into();
                },
            );
        }

        vec![]
    }

    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Esc => {
                        return vec![super::Message::Back];
                    }
                    KeyCode::Enter => {
                        let mut matches = self.nucleo.snapshot().matched_items(..);

                        if let Some(idx) = self.list_state.selected() {
                            let item = matches.nth(idx).unwrap();

                            let value = item.data.0.clone();
                            return vec![super::Message::Back, (self.map)(value)];
                        }

                        return vec![super::Message::Back];
                    }
                    KeyCode::Down => {
                        self.list_state.select_next();
                        return vec![];
                    }
                    KeyCode::Up => {
                        self.list_state.select_previous();
                        return vec![];
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

                vec![]
            }
            _ => vec![],
        }
    }
}
