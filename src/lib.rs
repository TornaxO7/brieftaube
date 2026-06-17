use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;

mod ui;

/// Stores the app state
#[derive(Debug, Default)]
struct State {
    ui: ui::UiState,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut state = State::new();

    loop {
        ui::render(&mut state, terminal)?;

        if crossterm::event::read()?.is_key_press() {
            break;
        }
    }

    Ok(())
}
