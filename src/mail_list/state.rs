use super::Action;
use crate::{
    backend::{
        mailbox::types::MailboxId,
        mails::{
            MailsBackend,
            types::{MailDisplay, MailEntryType, MailId, MailKeyword, MailUpdate, ThreadMarker},
        },
    },
    mail_list::widget::RenderData,
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use ratatui::widgets::TableState;
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

    in_mailbox: MailboxId,
    selected: HashSet<MailId>,
    backend: Rc<MailsBackend>,

    table_state: TableState,
}

impl State {
    pub fn new(in_mailbox: MailboxId, backend: Rc<MailsBackend>) -> Self {
        backend.init(in_mailbox.clone());

        Self {
            app_actions: Vec::with_capacity(2),
            overlay: None,
            backend,
            in_mailbox,

            table_state: TableState::new().with_selected(0),

            selected: HashSet::new(),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("j", Action::NavigateToNextMail),
                ("k", Action::NavigateToPreviousMail),
                ("h", Action::FoldThreadOrGoBack),
                ("l", Action::UnfoldThreadOrViewMail),
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

            Action::NavigateToNextMail => self.navigate_to_mail_below(),
            Action::NavigateToPreviousMail => self.navigate_to_mail_above(),
            Action::NavigateToTop => self.navigate_to_top(),
            Action::NavigateToBottom => self.navigate_to_bottom(),

            Action::ToggleMailSelection => self.toggle_mail_selection(),
            Action::MarkSelectedMailsAsUnseen => {
                self.change_keywords_of_mail(vec![(MailKeyword::Seen, false)])
            }
            Action::MarkSelectedMailsAsSeen => {
                self.change_keywords_of_mail(vec![(MailKeyword::Seen, true)])
            }

            Action::FoldThread => {
                self.fold_thread();
            }

            Action::FoldThreadOrGoBack => {
                if !self.fold_thread() {
                    self.apply_action(Action::Back);
                }
            }
            Action::UnfoldThreadOrViewMail => {
                if !self.unfold_thread() {
                    self.apply_action(Action::ViewSelectedMail);
                }
            }
            Action::UnfoldThread => {
                self.unfold_thread();
            }

            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
            }
            Action::ViewSelectedMail => self.view_selected_mail(),

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

impl State {
    fn navigate_to_mail_above(&mut self) {
        self.table_state.select_previous();
    }

    fn navigate_to_mail_below(&mut self) {
        self.table_state.select_next();
    }

    fn navigate_to_top(&mut self) {
        self.table_state.select_first();
    }

    fn navigate_to_bottom(&mut self) {
        self.table_state.select_last();
    }

    fn toggle_mail_selection(&mut self) {
        let Some(idx) = self.table_state.selected() else {
            return;
        };

        let Some(mails) = self.backend.get_mails(&self.in_mailbox) else {
            return;
        };

        let id = &mails[idx].id.clone();
        if !self.selected.remove(id) {
            self.selected.insert(id.clone());
        }

        self.navigate_to_mail_below();
    }

    fn change_keywords_of_mail(&mut self, patch: Vec<(MailKeyword, bool)>) {
        if self.selected.is_empty() {
            let Some(idx) = self.table_state.selected() else {
                return;
            };

            let Some(mails) = self.backend.get_mails(&self.in_mailbox) else {
                return;
            };

            let id = &mails[idx].id.clone();
            let update = MailUpdate {
                id: id.clone(),
                patch_keywords: Some(patch),
                ..Default::default()
            };

            self.backend.request_update_mails(vec![update]);
        } else {
            let updates: Vec<MailUpdate> = self
                .selected
                .drain()
                .map(|id| MailUpdate {
                    id,
                    patch_keywords: Some(patch.clone()),
                    ..Default::default()
                })
                .collect();

            self.backend.request_update_mails(updates);
        }
    }

    // return `true` if just unfolded
    fn unfold_thread(&mut self) -> bool {
        let Some(idx) = self.table_state.selected() else {
            return false;
        };

        let Some(mails) = self.backend.get_mails(&self.in_mailbox) else {
            return false;
        };

        let entry = &mails[idx];
        match entry.ty {
            MailEntryType::Root => self.backend.unfold_mail(&self.in_mailbox, &entry.id),
            MailEntryType::Child => false,
        }
    }

    fn fold_thread(&mut self) -> bool {
        let Some(idx) = self.table_state.selected() else {
            return false;
        };

        let Some(mails) = self.backend.get_mails(&self.in_mailbox) else {
            return false;
        };

        let entry = &mails[idx];
        match entry.ty {
            MailEntryType::Root => match mails.get(idx + 1) {
                Some(next_entry) => match next_entry.ty {
                    MailEntryType::Root => false,
                    MailEntryType::Child => {
                        self.backend.fold_thread(&self.in_mailbox, &entry.thread);
                        self.table_state.select(Some(idx));
                        true
                    }
                },
                None => false,
            },
            MailEntryType::Child => {
                // set the table state index
                match mails.iter().position(|other| other.thread == entry.thread) {
                    Some(root_pos) => {
                        tracing::debug!("Root pos: {}", root_pos);
                        self.table_state.select(Some(root_pos - 1));
                    }
                    None => self.table_state.select_first(),
                }

                self.backend.fold_thread(&self.in_mailbox, &entry.thread);

                true
            }
        }
    }

    fn view_selected_mail(&mut self) {
        let Some(mails) = self.backend.get_mails(&self.in_mailbox) else {
            return;
        };

        let Some(idx) = self.table_state.selected() else {
            return;
        };

        let mail_id = &mails[idx].id.clone();
        self.app_actions
            .push(crate::Action::OpenMailViewer(mail_id.clone()));
    }
}

// for widget
impl State {
    pub fn get_render_data<'a>(&'a mut self) -> Option<RenderData<'a>> {
        let entries = self.backend.get_mails(&self.in_mailbox)?;

        let rows = entries
            .iter()
            .filter_map(|entry| {
                let mail_id = &entry.id;
                let marker = match entry.ty {
                    MailEntryType::Root => ThreadMarker::Root,
                    MailEntryType::Child => ThreadMarker::Child,
                };

                let mail = self.backend.get_mail(mail_id)?;
                let row = MailDisplay::from((&mail, marker));
                Some(row)
            })
            .collect();

        Some(RenderData {
            rows,
            table_state: &mut self.table_state,
            selected: &self.selected,
        })
    }
}
