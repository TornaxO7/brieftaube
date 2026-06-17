use crate::State;
use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;

#[derive(Debug)]
enum Focus {
    MailboxList,
    MailList,
    Pager,
    Composer,
}

#[derive(Debug)]
pub struct UiState {
    focus: Focus,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            focus: Focus::MailList,
        }
    }
}

pub fn render(state: &mut State, terminal: &mut DefaultTerminal) -> Result<()> {
    Ok(())
}
