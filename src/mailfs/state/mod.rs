mod column_ctx;
mod input_type;
mod palette_value;

pub use palette_value::PaletteValue;
use ratatui::widgets::TableState;

use super::Action;
use crate::{
    backend::{Backend, types::CollapsedMail},
    mailfs::{state::column_ctx::ColumnCtxEntry, widget::RenderData},
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use column_ctx::ColumnCtx;
use input_type::InputType;
use std::{collections::HashMap, rc::Rc, vec::Drain};

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    overlay: Option<ScreenOverlay<PaletteValue, InputType>>,

    columns: Vec<ColumnCtx>,
    current_column: usize,

    backend: Rc<Backend>,
}

impl State {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            overlay: None,
            columns: vec![],
            current_column: 0,
            app_actions: Vec::with_capacity(2),
            keybindings: KeybindManager::new(HashMap::from([("q", Action::Quit)])),
        }
    }
}

impl<'a> ScreenState<'a, Action, PaletteValue, InputType, RenderData<'a>> for State {
    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.quit(),
            Action::OpenCommandPalette => self.open_command_palette(),
            Action::NavigateDown => self.navigate_down(),
            Action::NavigateUp => self.navigate_up(),
            Action::NavigateToTop => self.navigate_to_top(),
            Action::NavigateToBottom => self.navigate_to_bottom(),
            Action::ActivateSelectedEntry => self.activate_selected_entry(),
            Action::OpenLogs => self.open_logs(),
        }
    }

    fn get_app_actions(&mut self) -> Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteValue, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<PaletteValue, InputType>) {
        match result {
            ScreenOverlayResult::Palette(value) => match value {
                PaletteValue::Action(action) => self.apply_action(action),
            },
            ScreenOverlayResult::Cancel => {}
            ScreenOverlayResult::Input { .. } => unreachable!(),
        }
    }

    fn render_data(&'a mut self) -> RenderData<'a> {
        self.init_columns();

        let left = {
            todo!();
        };

        let center = {
            // TODO: Update column entries first, before preparing data
            let column = self.columns.get(self.current_column).unwrap();
            todo!()
        };

        let right = {
            todo!();
        };

        RenderData {
            left,
            center,
            right,
        }
    }
}

/// Helper functions
impl State {
    fn current_column_mut(&mut self) -> Option<&mut ColumnCtx> {
        self.columns.get_mut(self.current_column)
    }
}

/// Action implementations
impl State {
    fn quit(&mut self) {
        self.app_actions.push(crate::Action::Quit);
    }

    fn open_command_palette(&mut self) {
        self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
            Action::palette_options(),
        )));
    }

    fn navigate_down(&mut self) {
        if let Some(column) = self.current_column_mut() {
            column.state.select_next();
        }
    }

    fn navigate_up(&mut self) {
        if let Some(column) = self.current_column_mut() {
            column.state.select_previous();
        }
    }

    fn navigate_to_top(&mut self) {
        if let Some(column) = self.current_column_mut() {
            column.state.select_first();
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Some(column) = self.current_column_mut() {
            column.state.select_last();
        }
    }

    fn activate_selected_entry(&mut self) {
        todo!()
    }

    fn open_logs(&mut self) {
        self.app_actions.push(crate::Action::OpenLogViewer);
    }
}

impl<'a> State {
    fn init_columns(&mut self) -> Option<()> {
        self.init_column(self.current_column)?;
        self.init_column(self.current_column + 1)?;

        Some(())
    }

    /// Returns `None` if the column couldn't be initialised yet because the backend still needs fetch the data,
    /// otherwise `Some(())`.
    fn init_column(&mut self, column_idx: usize) -> Option<()> {
        let is_column_loaded = self.columns.get(column_idx).is_some();
        if !is_column_loaded {
            let mut entries: Vec<ColumnCtxEntry> = Vec::new();

            let parent_mailbox_id = if column_idx == 0 {
                None
            } else {
                self.columns[column_idx - 1].mailbox.clone()
            };

            // get mailboxes
            {
                let mailbox_ids = self
                    .backend
                    .get_child_mailboxes(parent_mailbox_id.clone())?;
                for mailbox_id in mailbox_ids {
                    entries.push(ColumnCtxEntry::Mailbox(mailbox_id));
                }
            }

            // get mails
            if let Some(parent_mailbox_id) = parent_mailbox_id.as_ref() {
                let collapsed_mails = self.backend.get_collapsed_mails(parent_mailbox_id)?;
                for collapsed_mail in collapsed_mails {
                    match collapsed_mail {
                        CollapsedMail::SingleMail(mail_id) => {
                            entries.push(ColumnCtxEntry::SingleMail(mail_id))
                        }
                        CollapsedMail::CollapsedThread(thread_id) => {
                            entries.push(ColumnCtxEntry::CollapsedThread(thread_id))
                        }
                    }
                }
            }

            let state = if entries.is_empty() {
                TableState::new()
            } else {
                TableState::new().with_selected(1)
            };

            self.columns.insert(
                column_idx,
                ColumnCtx {
                    mailbox: parent_mailbox_id,
                    entries,
                    state,
                },
            );
        }

        Some(())
    }
}
