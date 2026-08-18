use throbber_widgets_tui::ThrobberState;

#[derive(Debug, Clone)]
pub enum Loadable<T> {
    Loading(ThrobberState),
    Loaded(T),
}

impl<T> Loadable<T> {
    pub fn loading() -> Self {
        Self::Loading(ThrobberState::default())
    }

    pub fn as_ref(&self) -> Loadable<&T> {
        match self {
            Self::Loading(s) => Loadable::Loading(s.clone()),
            Self::Loaded(v) => Loadable::Loaded(v),
        }
    }

    pub fn map<U: std::fmt::Debug + Clone>(self, f: impl FnOnce(T) -> U) -> Loadable<U> {
        match self {
            Self::Loading(state) => Loadable::Loading(state),
            Self::Loaded(value) => Loadable::Loaded(f(value)),
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Loadable::Loaded(_))
    }

    pub fn loaded(&self) -> Option<&T> {
        match self {
            Self::Loading(_) => None,
            Self::Loaded(value) => Some(value),
        }
    }
}
