#[derive(Debug, Clone, Default)]
pub enum Loadable<T> {
    #[default]
    NotLoaded,
    Loading,
    Loaded(T),
    Error(String),
}

impl<T> Loadable<T> {
    pub fn as_ref(&self) -> Loadable<&T> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => Loadable::Loaded(value),
            Loadable::Error(message) => Loadable::Error(message.clone()),
        }
    }

    pub fn as_mut(&mut self) -> Loadable<&mut T> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => Loadable::Loaded(value),
            Loadable::Error(message) => Loadable::Error(message.clone()),
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Loadable<U> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => Loadable::Loaded(f(value)),
            Loadable::Error(message) => Loadable::Error(message),
        }
    }

    pub fn and_then<U>(self, f: impl FnOnce(T) -> Loadable<U>) -> Loadable<U> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => f(value),
            Loadable::Error(err) => Loadable::Error(err),
        }
    }

    pub fn loaded(&self) -> Option<&T> {
        match self {
            Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => None,
            Loadable::Loaded(value) => Some(value),
        }
    }
}
