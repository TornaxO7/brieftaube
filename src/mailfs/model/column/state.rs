use super::Column;
use throbber_widgets_tui::ThrobberState;
use tokio::sync::watch;

// TODO: Add intermediate state, which shows that it's loading
/// Internal representation of a column
#[derive(Clone, Debug, Default)]
pub enum ColumnState {
    #[default]
    NotLoaded,
    Loading {
        throbber: ThrobberState,
        notifier: watch::Receiver<()>,
    },
    Loaded(Column),
}

impl ColumnState {
    pub fn loading(notifier: watch::Receiver<()>) -> Self {
        Self::Loading {
            throbber: ThrobberState::default(),
            notifier,
        }
    }

    pub fn loaded(&self) -> Option<&Column> {
        match self {
            Self::NotLoaded | Self::Loading { .. } => None,
            Self::Loaded(column) => Some(column),
        }
    }

    pub fn loaded_mut(&mut self) -> Option<&mut Column> {
        match self {
            Self::NotLoaded | Self::Loading { .. } => None,
            Self::Loaded(column) => Some(column),
        }
    }
}
