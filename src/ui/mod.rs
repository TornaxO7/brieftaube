use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame};

mod composer;
mod mails;
mod pager;

#[derive(Debug, Clone, Copy)]
enum Focus {
    MailboxList,
    MailList,
    Pager,
    Composer,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Mails,
    Composer,
    Pager,
}

#[derive(Debug)]
pub struct UiState {
    focus: Focus,
    mode: Mode,

    mails: mails::State,
    pager: pager::State,
    composer: composer::State,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            focus: Focus::MailList,
            mode: Mode::Mails,

            mails: mails::State::default(),
            pager: pager::State::default(),
            composer: composer::State::default(),
        }
    }
}

pub fn render(state: &mut UiState, frame: &mut Frame) {
    match state.mode {
        Mode::Mails => mails::render(&mut state.mails, frame),
        Mode::Composer => composer::render(&state.composer, frame),
        Mode::Pager => pager::render(&state.pager, frame),
    }
}
