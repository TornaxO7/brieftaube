mod action;
mod mail_template;
mod state;

use crate::{
    backend::Account,
    ui::{
        command_palette::{CommandPalette, HandleEventResult},
        composer::mail_template::MailTemplate,
    },
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

        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area);

        render_mail_content(mail, top, buf);
        render_attachment_list(mail, bottom, buf);

        if let Some(cmd) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            cmd.palette.render(a, buf);
        }
    }
}

/// Rendering implementations
fn render_mail_content(mail: &MailTemplate, area: Rect, buf: &mut Buffer) {
    tracing::debug!("{:#?}", mail);
    let content = mail.renderable();
    Widget::render(Paragraph::new(content).block(Block::bordered()), area, buf)
}

fn render_attachment_list(_mail: &MailTemplate, _area: Rect, _buf: &mut Buffer) {
    // TODO
}

// #[derive(Debug)]
// struct AttachmentWidget<'a> {
//     name: &'a str,
//     ty: &'a str,
// }

// impl<'a> Widget for AttachmentWidget<'a> {
//     fn render(self, area: Rect, buf: &mut Buffer)
//     where
//         Self: Sized,
//     {
//         let [left, right] =
//             Layout::horizontal([Constraint::Fill(0), Constraint::Fill(0)]).areas(area);

//         Widget::render(Paragraph::new(self.name).left_aligned(), left, buf);
//         Widget::render(Paragraph::new(self.ty).right_aligned(), right, buf);
//     }
// }
