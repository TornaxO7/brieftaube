use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

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

impl Widget for &UiState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.mode {
            Mode::Mails => self.mails.render(area, buf),
            Mode::Composer => self.composer.render(area, buf),
            Mode::Pager => self.pager.render(area, buf),
        };
    }
}
