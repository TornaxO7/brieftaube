mod action;

use crate::ui::{
    palette::{self, CommandPalette},
    keybindmanager::KeybindManager,
};
pub use action::Action;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Clear, Widget},
};
use std::{collections::HashMap, str::FromStr};
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

#[derive(Debug)]
enum PaletteType {
    /// Palette is displaying commands
    Command,
}

#[derive(Debug)]
struct PaletteCtx {
    palette: CommandPalette,
    ty: PaletteType,
}

pub struct LogViewer {
    palette: Option<PaletteCtx>,
    keybindings: KeybindManager<super::Action>,

    state: TuiWidgetState,
    log_file_path: String,
    callback: Box<super::Action>,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            log_file_path: crate::get_log_file_path()
                .expect("Get log file path")
                .to_string_lossy()
                .to_string(),
            state: TuiWidgetState::new(),
            palette: None,
            callback: Box::new(super::Action::Quit),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit.into()),
                (":", Action::OpenCommandPalette.into()),
            ])),
        }
    }

    pub fn set_callback(&mut self, callback: Box<super::Action>) {
        self.callback = callback;
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        if let Some(command_palette) = &mut self.palette {
            let mut actions = Vec::new();
            if let Some(result) = command_palette.palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    palette::HandleEventResult::Cancel => {}
                    palette::HandleEventResult::Selected(value) => {
                        match command_palette.ty {
                            PaletteType::Command => {
                                actions.push(Action::from_str(&value).unwrap().into())
                            }
                        }
                    }
                }
            }

            return actions;
        }

        match self.keybindings.handle_event(event) {
            Some(action) => vec![action],
            None => vec![],
        }
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        tracing::debug!("Action: {:?}", a);
        match a {
            Action::Back => return Some(*self.callback.clone()),
            Action::Quit => return Some(super::Action::Quit),

            Action::CloseCommandPalette => self.palette = None,
            Action::OpenCommandPalette => {
                self.palette = Some(PaletteCtx {
                    palette: CommandPalette::new(Action::palette_options()),
                    ty: PaletteType::Command,
                })
            }
        }

        None
    }
}

impl std::fmt::Debug for LogViewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogViewer")
            .field("palette", &self.palette)
            .field("keybindings", &self.keybindings)
            .finish()
    }
}

impl Widget for &mut LogViewer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        TuiLoggerWidget::default()
            .block(Block::bordered().title(self.log_file_path.as_str()))
            .style_error(Style::default().red())
            .style_warn(Style::default().yellow())
            .style_info(Style::default().green())
            .output_file(false)
            .output_line(false)
            .output_target(false)
            .output_timestamp(Some("[%H:%M:%S]".to_string()))
            .state(&self.state)
            .render(area, buf);

        if let Some(command_palette) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            command_palette.palette.render(a, buf);
        }
    }
}
