pub mod keybindmanager;

#[derive(Debug, Clone, Default)]
pub enum Loadable<T> {
    #[default]
    NotLoaded,
    Loading,
    Loaded(T),
    Error,
}

impl<T> Loadable<T> {
    pub fn as_ref(&self) -> Loadable<&T> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => Loadable::Loaded(value),
            Loadable::Error => Loadable::Error,
        }
    }

    pub fn as_mut(&mut self) -> Loadable<&mut T> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => Loadable::Loaded(value),
            Loadable::Error => Loadable::Error,
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Loadable<U> {
        match self {
            Loadable::NotLoaded => Loadable::NotLoaded,
            Loadable::Loading => Loadable::Loading,
            Loadable::Loaded(value) => Loadable::Loaded(f(value)),
            Loadable::Error => Loadable::Error,
        }
    }
}
