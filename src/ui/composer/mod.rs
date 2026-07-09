mod action;
// mod mail_template;
mod state;

use crate::{
    backend::Account,
    ui::{keybindmanager::KeybindManager, palette},
};
pub use action::Action;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
enum PaletteType {
    /// Palette is displaying commands
    Command(Action),
}

pub struct Composer {
    state: state::State,
    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<super::Action>,
}

impl Composer {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            state: state::State::new(account),
            palette: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::ScrollDown.into()),
                ("k", Action::ScrollUp.into()),
                ("q", super::Action::Quit),
                ("h", super::Action::OpenMailList(None)),
                ("e", Action::OpenMailInEditor.into()),
                (":", Action::OpenCommandPalette.into()),
            ])),
        }
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        if let Some(palette) = &mut self.palette {
            let mut actions = Vec::new();
            if let Some(result) = palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    palette::HandleEventResult::Cancel => {}
                    palette::HandleEventResult::Selected(value) => match value {
                        PaletteType::Command(action) => {
                            actions.push(action.into());
                        }
                    },
                };
            }

            return actions;
        }

        match self.keybindings.handle_event(event) {
            Some(action) => vec![action],
            None => vec![],
        }
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        match a {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(action::palette_options()));
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

        if let Some(state) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            StatefulWidget::render(palette::Palette::new(), a, buf, state);
        }
    }
}

/// Rendering implementations
fn render_mail_content(mail: &str, area: Rect, buf: &mut Buffer) {
    tracing::debug!("{:#?}", mail);
    Widget::render(Paragraph::new(mail).block(Block::bordered()), area, buf)
}
