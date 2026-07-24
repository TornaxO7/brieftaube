mod column_ctx;
mod input_type;
mod palette_value;

pub use palette_value::PaletteValue;

use super::Action;
use crate::{
    backend::Backend,
    mailfs::widget::RenderData,
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
        backend.request_mailboxes();

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
        self.sync_columns();

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
    fn sync_columns(&mut self) {
        let is_current_column_loaded = self.columns.get(self.current_column).is_some();
        if !is_current_column_loaded {
            let is_root = self.current_column == 0;
            let mailboxes = if is_root {
                self.backend.get_child_mailboxes(&None)
            } else {
                todo!();
            }
        }

        let is_right_column_loaded = self.columns.get(self.current_column + 1).is_some();
        if !is_right_column_loaded {
            // load column
        }
    }
}
