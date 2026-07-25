mod column_state;
mod error;
mod input_type;
mod palette_value;

pub use palette_value::PaletteValue;

use super::Action;
use crate::{
    backend::{
        Backend,
        mailbox::types::{ParentMailboxId, TOP_PARENT_MAILBOX_ID},
    },
    mailfs::widget::{ColumnDisplay, RenderData, RightColumn},
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

    columns: Vec<ColumnState>,
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
        let _ = self.load_current_and_next_column();

        let (left_part, rest) = self.columns.split_at_mut(self.current_column);
        let (center_part, right_part) = rest.split_at_mut(1.min(rest.len()));

        let left_column = left_part.as_mut().last_mut();
        let center_column = center_part.first_mut().unwrap();
        let right_column = right_part.first_mut();

        let left = left_column.map(|column| ColumnDisplay::new(column, self.backend.clone()));
        let right = match &center_column {
            ColumnState::Loading { .. } => None,
            ColumnState::Loaded { entries, state, .. } => match state.selected() {
                None => None,
                Some(idx) => match &entries[idx] {
                    ColumnStateEntry::Mailbox(_) => right_column.map(|column| {
                        RightColumn::ColumnData(ColumnDisplay::new(column, self.backend.clone()))
                    }),
                    ColumnStateEntry::SingleMail(_)
                    | ColumnStateEntry::CollapsedThread(_)
                    | ColumnStateEntry::UncollapsedThread(_) => None,
                },
            },
        };
        let center = ColumnDisplay::new(center_column, self.backend.clone());

        RenderData {
            left,
            center,
            right,
        }
    }
}

/// Helper functions
impl State {
    fn current_column_mut(&mut self) -> Option<&mut ColumnState> {
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
            match column {
                ColumnState::Loading { .. } => {}
                ColumnState::Loaded { state, .. } => state.select_next(),
            }
        }
    }

    fn navigate_up(&mut self) {
        if let Some(column) = self.current_column_mut() {
            // column.state.select_previous();
        }
    }

    fn navigate_to_top(&mut self) {
        if let Some(column) = self.current_column_mut() {
            // column.state.select_first();
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Some(column) = self.current_column_mut() {
            // column.state.select_last();
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
    fn load_current_and_next_column(&mut self) -> Result<(), error::BackendNotReady> {
        if self.columns.len() <= self.current_column + 1 {
            self.columns
                .resize_with(self.current_column + 2, || ColumnState::loading());
        }

        self.load_column(self.current_column)?;
        self.load_column(self.current_column + 1)?;
        Ok(())
    }

    fn load_column(&mut self, column_idx: usize) -> Result<(), error::BackendNotReady> {
        let mailbox = self.get_mailbox_id(column_idx);

        let column = &mut self.columns[column_idx];
        match column {
            ColumnState::Loaded { .. } => Ok(()),
            ColumnState::Loading { state } => {
                if let Some(mailbox) = mailbox {
                    let column_entries =
                        ColumnStateEntry::create_entries(mailbox.clone(), self.backend.clone())?;
                    *column = ColumnState::loaded(mailbox.clone(), column_entries);
                    return Ok(());
                }
                state.calc_next();
                Err(error::BackendNotReady)
            }
        }
    }

    /// Returns the mailbox-id which the given column should represent
    fn get_mailbox_id(&self, column_idx: usize) -> Option<ParentMailboxId> {
        let is_in_root_mailbox = column_idx == 0;

        if is_in_root_mailbox {
            Some(TOP_PARENT_MAILBOX_ID)
        } else {
            match self.columns.get(column_idx - 1).unwrap() {
                ColumnState::Loading { .. } => None,
                ColumnState::Loaded { entries, state, .. } => {
                    state.selected().and_then(|idx| match &entries[idx] {
                        ColumnStateEntry::Mailbox(id) => Some(Some(id.clone())),
                        _ => Some(None),
                    })
                }
            }
        }
    }
}
