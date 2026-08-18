use tokio::sync::watch;

#[derive(Debug, Clone, Default)]
pub enum RemoteData<T> {
    #[default]
    NotRequested,
    Requested {
        notifier: watch::Receiver<()>,
    },
    Loaded(T),
}

impl<T> RemoteData<T> {
    pub fn requesting(notifier: watch::Receiver<()>) -> Self {
        Self::Requested { notifier }
    }

    pub fn not_requested(&self) -> bool {
        matches!(self, Self::NotRequested)
    }

    pub fn as_ref(&self) -> RemoteData<&T> {
        match self {
            Self::NotRequested => RemoteData::NotRequested,
            Self::Requested { notifier: rx } => RemoteData::Requested {
                notifier: rx.clone(),
            },
            Self::Loaded(v) => RemoteData::Loaded(v),
        }
    }

    pub fn map<U: std::fmt::Debug + Clone>(self, f: impl FnOnce(T) -> U) -> RemoteData<U> {
        match self {
            Self::NotRequested => RemoteData::NotRequested,
            Self::Requested { notifier: rx } => RemoteData::Requested { notifier: rx },
            Self::Loaded(value) => RemoteData::Loaded(f(value)),
        }
    }

    pub fn loaded(&self) -> Option<&T> {
        match self {
            Self::NotRequested | Self::Requested { .. } => None,
            Self::Loaded(value) => Some(value),
        }
    }
}
