mod state;

pub use state::*;

use throbber_widgets_tui::ThrobberState;

/// Internal representation of a column
#[derive(Clone, Debug)]
pub enum ColumnState {
    Loading(ThrobberState),
    Loaded(Column),
}

impl ColumnState {
    pub fn loading() -> Self {
        Self::Loading(ThrobberState::default())
    }

    pub fn loaded(&self) -> Option<&Column> {
        match self {
            Self::Loading(_) => None,
            Self::Loaded(column) => Some(column),
        }
    }

    pub fn loaded_mut(&mut self) -> Option<&mut Column> {
        match self {
            Self::Loading(_) => None,
            Self::Loaded(column) => Some(column),
        }
    }
}
