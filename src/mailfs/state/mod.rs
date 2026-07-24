mod column_ctx;
mod input_type;
mod palette_value;

pub use palette_value::PaletteValue;

use super::Action;
use crate::{
    backend::{Backend, mailbox::MailboxBackend, mails::MailsBackend},
    config::Config,
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
        Self {
            backend,
            overlay: None,
            columns: vec![ColumnCtx::default()],
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

    fn render_data(&'a mut self) -> Option<RenderData<'a>> {
        let center = {
            let current_column = self.current_column_mut();
        };

        None
        // Some(RenderData {
        //     left: None,
        //     center: ColumnData {
        //         entries: vec![],
        //         state: &mut self.current_column_mut().state,
        //     },
        //     right: None,
        // })
    }
}

/// Helper functions
impl State {
    fn current_column_mut(&mut self) -> &mut ColumnCtx {
        self.columns
            .get_mut(self.current_column)
            .expect("Column exists")
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
        self.current_column_mut().state.select_next();
    }

    fn navigate_up(&mut self) {
        self.current_column_mut().state.select_previous();
    }

    fn navigate_to_top(&mut self) {
        self.current_column_mut().state.select_first();
    }

    fn navigate_to_bottom(&mut self) {
        self.current_column_mut().state.select_last();
    }

    fn activate_selected_entry(&mut self) {
        todo!()
    }

    fn open_logs(&mut self) {
        self.app_actions.push(crate::Action::OpenLogViewer);
    }
}
