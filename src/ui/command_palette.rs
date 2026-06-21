use std::{fmt::Display, marker::PhantomData, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent};
use nucleo::Nucleo;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};
use ratatui_textarea::TextArea;
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

type CommandName = String;
type CommandDescription = String;

pub trait CommandPaletteEntry:
    IntoEnumIterator + EnumMessage + EnumProperty + Display + std::fmt::Debug
{
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: CommandName,
    pub description: CommandDescription,
}

impl Command {
    pub fn new<S: ToString>(name: S, description: S) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HandleEventResult {
    Selected(CommandName),
    Quit,
}

pub struct CommandPalette<E: CommandPaletteEntry> {
    input: TextArea<'static>,
    nucleo: Nucleo<(CommandName, CommandDescription)>,

    list_state: ListState,
    _phantom: PhantomData<E>,
}

impl<E: CommandPaletteEntry> std::fmt::Debug for CommandPalette<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").field("input", &self.input).finish()
    }
}

impl<E: CommandPaletteEntry> CommandPalette<E> {
    pub fn new() -> Self {
        let nucleo: Nucleo<(String, String)> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 2);

        let inj = nucleo.injector();
        for c in E::iter() {
            if let Some(is_intern) = c.get_bool("intern") {
                if is_intern {
                    continue;
                }
            }

            let name = c.to_string();
            let description = c.get_message().unwrap().to_string();

            inj.push((name, description), |&(ref name, ref description), row| {
                row[0] = (*name).clone().into();
                row[1] = (*description).clone().into();
            });
        }

        Self {
            input: TextArea::default(),
            nucleo,
            list_state: ListState::default().with_selected(Some(0)),
            _phantom: PhantomData,
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<HandleEventResult> {
        match event.code {
            KeyCode::Esc => {
                self.reset();
                return Some(HandleEventResult::Quit);
            }
            KeyCode::Enter => {
                let result = {
                    let mut matches = self.nucleo.snapshot().matched_items(..);

                    if let Some(idx) = self.list_state.selected() {
                        HandleEventResult::Selected(matches.nth(idx).unwrap().data.0.clone())
                    } else {
                        HandleEventResult::Quit
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

impl<E: CommandPaletteEntry> Widget for &mut CommandPalette<E> {
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
                        .block(Block::bordered().title("Description")),
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
                .map(|output| output.data.1.as_str())
                .collect();

            StatefulWidget::render(
                List::new(options_content)
                    .block(Block::bordered().title("Commands"))
                    .highlight_style(Style::new().blue())
                    .direction(ListDirection::TopToBottom),
                options,
                buf,
                &mut self.list_state,
            );
        }
    }
}
