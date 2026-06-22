use std::sync::Arc;

use crossterm::event::KeyEvent;
use jmap_client::client::Client;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

mod command_palette;
// mod composer;
mod mails;
// mod pager;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,

    MailList(mails::Action),
    OpenMailList,
    // OpenComposer,
    // OpenPager,
}

impl From<mails::Action> for Action {
    fn from(action: mails::Action) -> Self {
        Self::MailList(action)
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Mails,
    // Composer,
    // Pager,
}

#[derive(Debug)]
pub struct State {
    mode: Mode,

    mails: mails::Mails,
    // pager: pager::State,
    // composer: composer::State,
}

impl State {
    pub async fn new(client: Arc<Client>) -> Self {
        Self {
            mode: Mode::Mails,

            mails: mails::Mails::new(client).await,
            // pager: pager::State::new(),
            // composer: composer::State::new(),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        let sub_actions = match self.mode {
            Mode::Mails => self.mails.handle_event(event),
            // Mode::Composer => self.composer.handle_event(event),
            // Mode::Pager => self.pager.handle_event(event),
        };

        for action in sub_actions {
            self.apply_action(action);
        }

        None
    }

    fn apply_action(&mut self, action: Action) -> Option<super::Action> {
        match action {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenMailList => self.mode = Mode::Mails,
            Action::MailList(action) => {
                return self
                    .mails
                    .apply_action(action)
                    .and_then(|action| self.apply_action(action));
            } // Action::OpenComposer => {
              //     self.mode = Mode::Composer;
              //     None
              // }
              // Action::OpenPager => {
              //     self.mode = Mode::Pager;
              //     None
              // }
        }

        None
    }
}

impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.mode {
            Mode::Mails => self.mails.render(area, buf),
            // Mode::Pager => self.pager.render(area, buf),
            // Mode::Composer => self.composer.render(area, buf),
        }
    }
}
