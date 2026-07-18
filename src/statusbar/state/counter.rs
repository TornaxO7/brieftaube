use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};
use tokio::sync::Notify;
use tracing::{Level, Subscriber};
use tracing_subscriber::Layer;

#[derive(Debug, Clone)]
pub struct Counter {
    pub errors: Arc<AtomicU16>,
    pub warnings: Arc<AtomicU16>,
    pub infos: Arc<AtomicU16>,

    has_changed: Arc<Notify>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            errors: Arc::new(AtomicU16::new(0)),
            warnings: Arc::new(AtomicU16::new(0)),
            infos: Arc::new(AtomicU16::new(0)),

            has_changed: Arc::new(Notify::new()),
        }
    }

    pub async fn has_changed(&self) {
        self.has_changed.notified().await
    }

    pub fn reset(&mut self) {
        self.errors.store(0, Ordering::Release);
        self.warnings.store(0, Ordering::Release);
        self.infos.store(0, Ordering::Release);
    }
}

impl<S: Subscriber> Layer<S> for Counter {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match *event.metadata().level() {
            Level::ERROR => {
                self.errors.fetch_add(1, Ordering::Relaxed);
                self.has_changed.notify_waiters();
            }
            Level::WARN => {
                self.warnings.fetch_add(1, Ordering::Relaxed);
                self.has_changed.notify_waiters();
            }
            Level::INFO => {
                self.infos.fetch_add(1, Ordering::Relaxed);
                self.has_changed.notify_waiters();
            }
            _ => {}
        }
    }
}
