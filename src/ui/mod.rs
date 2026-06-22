use std::sync::Arc;

use crossterm::event::KeyEvent;
use jmap_client::client::Client;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

mod command_palette;
// mod composer;
mod mails;
// mod pager;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,

    OpenMailList,
    // OpenComposer,
    // OpenPager,
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
        match self.mode {
            Mode::Mails => self.mails.handle_event(event),
            // Mode::Composer => self.composer.handle_event(event),
            // Mode::Pager => self.pager.handle_event(event),
        }
        .and_then(|a| match a {
            Action::Quit => Some(super::Action::Quit),
            Action::OpenMailList => {
                self.mode = Mode::Mails;
                None
            } // Action::OpenComposer => {
              //     self.mode = Mode::Composer;
              //     None
              // }
              // Action::OpenPager => {
              //     self.mode = Mode::Pager;
              //     None
              // }
        })
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
