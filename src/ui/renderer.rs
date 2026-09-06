use ratatui::DefaultTerminal;
use tokio::sync::watch;

pub enum RendererFrame {}

pub struct Renderer {
    terminal: DefaultTerminal,
    rx: watch::Receiver<RendererFrame>,
}

impl Renderer {
    pub fn new(rx: watch::Receiver<RendererFrame>) -> Self {
        Self {
            terminal: ratatui::init(),
            rx,
        }
    }

    pub async fn run(mut self) {
        while let Ok(frame_data) = self.rx.changed().await {
            todo!()
        }
    }
}
