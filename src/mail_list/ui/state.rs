use super::Action;
use crate::{
    mail_list::backend::MailListBackend,
    utils::{
        EmailKeyword, MailId,
        ui::{
            ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager,
            palette,
        },
    },
};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

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

    pub selected: HashSet<MailId>,
    pub backend: Rc<MailListBackend>,
}

impl State {
    pub fn new(backend: Rc<MailListBackend>) -> Self {
        backend.init();

        Self {
            app_actions: Vec::with_capacity(2),
            overlay: None,
            backend,

            selected: HashSet::new(),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("j", Action::NavigateToNextMail),
                ("k", Action::NavigateToPreviousMail),
                ("h", Action::FoldThread),
                ("l", Action::UnfoldThread),
                ("<BS>", Action::Back),
                ("<C-l>", Action::OpenLogs),
                ("gg", Action::NavigateToTop),
                ("ge", Action::NavigateToBottom),
                (" ", Action::ToggleMailSelection),
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

            Action::NavigateToNextMail => self.backend.navigate_to_next_mail(),
            Action::NavigateToPreviousMail => self.backend.navigate_to_previous_mail(),
            Action::NavigateToTop => self.backend.navigate_to_top(),
            Action::NavigateToBottom => self.backend.navigate_to_bottom(),

            Action::ToggleMailSelection => {
                if let Some(mail) = self.backend.get_selected_mail() {
                    if !self.selected.remove(&mail.id) {
                        self.selected.insert(mail.id);
                    }
                    self.backend.navigate_to_next_mail();
                }
            }
            Action::MarkSelectedMailsAsUnseen => {
                if self.selected.is_empty() {
                    if let Some(mail) = self.backend.get_selected_mail() {
                        self.backend.update_keywords(
                            vec![mail.id],
                            HashSet::from([EmailKeyword::Seen]),
                            false,
                        );
                    }
                } else {
                    let ids: Vec<MailId> = self.selected.drain().collect();
                    self.backend
                        .update_keywords(ids, HashSet::from([EmailKeyword::Seen]), false);
                }
            }
            Action::MarkSelectedMailsAsSeen => {
                if self.selected.is_empty() {
                    if let Some(mail) = self.backend.get_selected_mail() {
                        self.backend.update_keywords(
                            vec![mail.id],
                            HashSet::from([EmailKeyword::Seen]),
                            true,
                        );
                    }
                } else {
                    let ids: Vec<MailId> = self.selected.drain().collect();
                    self.backend
                        .update_keywords(ids, HashSet::from([EmailKeyword::Seen]), true);
                }
            }

            Action::FoldThread => {
                if let Some(pos) = self.backend.get_selected_mail_position() {
                    self.backend.fold_thread(pos);
                }
            }
            Action::UnfoldThread => {
                if let Some(pos) = self.backend.get_selected_mail_position() {
                    self.backend.unfold_thread(pos);
                }
            }

            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
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
