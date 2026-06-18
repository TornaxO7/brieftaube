use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

mod command_palette;
mod composer;
mod mails;
mod pager;

#[derive(Debug, Clone, Copy)]
enum Action {
    Quit,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Mails,
    Composer,
    Pager,
}

#[derive(Debug)]
pub struct State {
    mode: Mode,

    mails: mails::State,
    pager: pager::State,
    composer: composer::State,
}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        let action = match self.mode {
            Mode::Mails => self.mails.handle_event(event),
            Mode::Composer => self.composer.handle_event(event),
            Mode::Pager => self.pager.handle_event(event),
        };

        action.map(|a| match a {
            Action::Quit => super::Action::Quit,
        })
    }
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.mode {
            Mode::Mails => self.mails.render(area, buf),
            Mode::Pager => self.pager.render(area, buf),
            Mode::Composer => self.composer.render(area, buf),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Mails,

            mails: mails::State::default(),
            pager: pager::State::default(),
            composer: composer::State::default(),
        }
    }
}
