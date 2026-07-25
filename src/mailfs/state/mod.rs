mod column_state;
mod error;
mod input_type;
mod palette_value;

pub use palette_value::PaletteValue;
use tracing::{debug, instrument};

use super::Action;
use crate::{
    backend::{
        Backend,
        mailbox::types::{ParentMailboxId, TOP_PARENT_MAILBOX_ID},
    },
    mailfs::widget::{ColumnDisplay, MailPreview, RenderData, RightColumn},
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
pub use column_state::{ColumnState, ColumnStateEntry};
use input_type::InputType;
use std::{collections::HashMap, rc::Rc, vec::Drain};

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    overlay: Option<ScreenOverlay<PaletteValue, InputType>>,

    columns: HashMap<ParentMailboxId, ColumnState>,
    selection_stack: Vec<ParentMailboxId>,

    backend: Rc<Backend>,
}

impl State {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            overlay: None,
            columns: HashMap::new(),
            selection_stack: vec![TOP_PARENT_MAILBOX_ID],
            app_actions: Vec::with_capacity(2),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("j", Action::NavigateDown),
                ("k", Action::NavigateUp),
                ("gg", Action::NavigateToTop),
                ("ge", Action::NavigateToBottom),
            ])),
        }
    }
}

impl<'a> ScreenState<'a, Action, PaletteValue, InputType, RenderData<'a>> for State {
    #[instrument(skip(self))]
    fn apply_action(&mut self, action: Action) {
        debug!("{:?}", action);
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
        self.update_columns();
        let backend = self.backend.clone();

        let right_preview = self
            .get_center_column()
            .and_then(|center_column| center_column.selected_entry())
            .and_then(|selected_entry| match selected_entry {
                ColumnStateEntry::Mailbox(_) => None,
                ColumnStateEntry::SingleMail(id)
                | ColumnStateEntry::CollapsedThread(id, _)
                | ColumnStateEntry::ThreadStart(id, _)
                | ColumnStateEntry::ThreadChild(id, _)
                | ColumnStateEntry::ThreadEnd(id, _) => Some(id),
            })
            .and_then(|mail_id| backend.mail_get_data(mail_id))
            .map(MailPreview::from)
            .map(RightColumn::MailPreview);

        let (left, center, right_column) = {
            let backend = self.backend.clone();
            let center_mailbox = self.get_center_column_mailbox();
            match (
                self.get_left_column_mailbox(),
                self.get_right_column_mailbox(),
            ) {
                (Some(left_mailbox), Some(right_mailbox)) => {
                    let [left, center, right] = self.columns.get_disjoint_mut([
                        &left_mailbox,
                        &center_mailbox,
                        &right_mailbox,
                    ]);

                    (
                        ColumnDisplay::new(left, backend.clone()),
                        ColumnDisplay::new(center, backend.clone()),
                        ColumnDisplay::new(right, backend.clone()),
                    )
                }
                (Some(left_mailbox), None) => {
                    let [left, center] = self
                        .columns
                        .get_disjoint_mut([&left_mailbox, &center_mailbox]);

                    (
                        ColumnDisplay::new(left, backend.clone()),
                        ColumnDisplay::new(center, backend.clone()),
                        None,
                    )
                }
                (None, Some(right_mailbox)) => {
                    let [center, right] = self
                        .columns
                        .get_disjoint_mut([&center_mailbox, &right_mailbox]);

                    (
                        None,
                        ColumnDisplay::new(center, backend.clone()),
                        ColumnDisplay::new(right, backend.clone()),
                    )
                }
                (None, None) => (
                    None,
                    ColumnDisplay::new(self.get_center_column_mut(), backend.clone()),
                    None,
                ),
            }
        };

        let right = right_preview.or(right_column.map(|column| RightColumn::ColumnData(column)));

        RenderData {
            left,
            center,
            right,
        }
    }
}

/// Helper functions
impl State {
    fn get_left_column_mailbox(&self) -> Option<ParentMailboxId> {
        (self.selection_stack.len().checked_sub(2))
            .map(|idx| &self.selection_stack[idx])
            .cloned()
    }

    fn get_center_column_mailbox(&self) -> ParentMailboxId {
        self.selection_stack.last().cloned().unwrap()
    }

    fn get_right_column_mailbox(&self) -> Option<ParentMailboxId> {
        self.get_center_column()
            .and_then(|center| center.selected_entry())
            .cloned()
            .map(|selected| {
                if let ColumnStateEntry::Mailbox(id) = selected {
                    Some(id)
                } else {
                    None
                }
            })
    }

    fn get_center_column(&self) -> Option<&ColumnState> {
        self.columns.get(self.selection_stack.last().unwrap())
    }

    fn get_center_column_mut(&mut self) -> Option<&mut ColumnState> {
        self.columns.get_mut(self.selection_stack.last().unwrap())
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
        if let Some(column) = self.get_center_column_mut() {
            column.state.select_next();
        }
    }

    fn navigate_up(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            column.state.select_previous();
        }
    }

    fn navigate_to_top(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            column.state.select_first();
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
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

/// Methods for loading the columns
impl<'a> State {
    fn update_columns(&mut self) {
        if self.get_center_column().is_none() {
            let current_mailbox = self.selection_stack.last().unwrap();

            if let Ok(entries) =
                ColumnStateEntry::create_entries(current_mailbox.clone(), self.backend.clone())
            {
                let state = ColumnState::new(current_mailbox.clone(), entries);
                self.columns.insert(current_mailbox.clone(), state);
            }
        }

        if let Some(current) = self.get_center_column() {
            if let Some(entry) = current.selected_entry() {
                if let ColumnStateEntry::Mailbox(id) = entry {
                    if let Ok(entries) =
                        ColumnStateEntry::create_entries(Some(id.clone()), self.backend.clone())
                    {
                        let state = ColumnState::new(Some(id.clone()), entries);
                        self.columns.insert(Some(id.clone()), state);
                    }
                }
            }
        }
    }
}
