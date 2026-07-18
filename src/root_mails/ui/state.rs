use super::Action;
use crate::{
    root_mails::backend::RootMailsBackend,
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {}

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,

    pub backend: Rc<RootMailsBackend>,
}

impl State {
    pub fn new(backend: Rc<RootMailsBackend>) -> Self {
        backend.init();

        Self {
            app_actions: Vec::with_capacity(2),
            overlay: None,
            backend,
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("j", Action::SelectNextMail),
                ("k", Action::SelectPreviousMail),
                ("h", Action::Back),
                ("l", Action::OpenThread),
            ])),
        }
    }
}

impl ScreenState<Action, PaletteType, InputType> for State {
    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::Back => self.app_actions.push(crate::Action::Back),

            Action::SelectNextMail => self.backend.select_next_mail(),
            Action::SelectPreviousMail => self.backend.select_previous_mail(),

            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
            }
            Action::OpenThread => {
                todo!()
            }
            Action::ViewSelectedMail => {
                todo!()
            }
            Action::ComposeMail => {
                self.app_actions.push(crate::Action::OpenComposer);
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut crate::utils::ui::ScreenOverlay<PaletteType, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(
        &mut self,
        result: crate::utils::ui::ScreenOverlayResult<PaletteType, InputType>,
    ) {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Cancel => {}
            ScreenOverlayResult::Palette(value) => match value {
                PaletteType::Action(action) => {
                    self.apply_action(action);
                }
            },
            ScreenOverlayResult::Input { value: _, typ: _ } => {
                unreachable!("Sus")
            }
        }
    }
}
