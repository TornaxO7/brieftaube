mod action;
// mod mail_template;
mod state;

use crate::{
    backend::Account,
    ui::command_palette::{CommandPalette, HandleEventResult},
};
pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, Paragraph, Widget},
};
use std::{str::FromStr, sync::Arc};

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

#[derive(Debug)]
pub struct Composer {
    state: state::State,
    palette: Option<PaletteCtx>,
}

impl Composer {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            state: state::State::new(account),
            palette: None,
        }
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        let mut actions = Vec::new();

        if let Some(command_palette) = &mut self.palette {
            if let Some(result) = command_palette.palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    HandleEventResult::Cancel => {}
                    HandleEventResult::Selected(value) => match command_palette.ty {
                        PaletteType::Command => {
                            actions.push(Action::from_str(&value).unwrap().into())
                        }
                    },
                };
            }

            return actions;
        }

        match event.code {
            KeyCode::Char('j') => actions.push(Action::ScrollDown.into()),
            KeyCode::Char('k') => actions.push(Action::ScrollUp.into()),
            KeyCode::Char('q') => actions.push(super::Action::Quit),
            KeyCode::Char('h') => actions.push(super::Action::OpenMailList(None)),
            KeyCode::Char('e') => actions.push(Action::OpenMailInEditor.into()),
            KeyCode::Char(':') => actions.push(Action::OpenCommandPalette.into()),
            _ => {}
        };

        actions
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        match a {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(PaletteCtx {
                    palette: CommandPalette::new(Action::palette_options()),
                    ty: PaletteType::Command,
                })
            }
            Action::CloseCommandPalette => self.palette = None,
            Action::ScrollUp => self.state.scroll_up(),
            Action::ScrollDown => self.state.scroll_down(),

            Action::OpenMailList => return Some(super::Action::OpenMailList(None)),
            Action::OpenMailInEditor => self.state.open_mail_in_editor(),
            Action::SendMail => {
                self.state.send_mail();
                self.reset();
                return Some(super::Action::OpenMailboxList);
            }
        }

        None
    }
}

impl Widget for &mut Composer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.state.update();

        let mail = self.state.get_mail();

        let [top, _bottom] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area);

        render_mail_content(mail, top, buf);
        // render_attachment_list(mail, bottom, buf);

        if let Some(cmd) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            cmd.palette.render(a, buf);
        }
    }
}

/// Rendering implementations
fn render_mail_content(mail: &str, area: Rect, buf: &mut Buffer) {
    tracing::debug!("{:#?}", mail);
    Widget::render(Paragraph::new(mail).block(Block::bordered()), area, buf)
}
