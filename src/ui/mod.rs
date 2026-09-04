// pub mod composer;
// pub mod log_viewer;
pub mod mailfs;
// pub mod palette;
// pub mod prompt;
// pub mod reader;
// pub mod statusbar;
mod utils;

use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use tracing::error;

enum Layer {
    Mailfs(mailfs::State),
}

impl Layer {
    pub fn is_overlay(&self) -> bool {
        false
    }
}

pub enum Action {
    Redraw,
    Quit,
}

/// Stores the app state
pub struct Ui {
    is_running: bool,
    layers: Vec<Layer>,
    needs_full_redraw: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            is_running: true,
            layers: vec![],
            needs_full_redraw: false,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();

        while self.is_running {
            tokio::select! {
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => if let Some(action) = self.handle_event(event) {
                        self.apply_action(action);
                    }
                    Some(Err(e)) => error!("{}", e),
                    None => {},
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {}

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        None
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Redraw => {
                self.needs_full_redraw = true;
            }

            Action::Quit => {
                self.is_running = false;
            }
        }
    }
}

pub trait LayerCore<ParentAction = Action> {
    fn handle_event(&mut self, event: Event) -> Option<ParentAction>;

    fn is_overlay(&self) -> bool {
        false
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect);
}

pub trait LayerOverlay: LayerCore {
    fn into_message(self) -> Option<String>;
}

pub trait LayerState<Action, ParentAction = Action>: LayerCore<ParentAction> {
    #[must_use]
    fn apply_action(&mut self, action: Action) -> Option<ParentAction>;

    #[must_use]
    fn handle_overlay<O>(&mut self, overlay: O) -> Option<ParentAction>
    where
        O: LayerOverlay;
}

// pub trait LayerModelDefaultHandleEvent<Action, ParentAction = crate::Action>:
//     LayerModel<Action, ParentAction>
// where
//     Action: Clone,
// {
//     fn keybinding_manager(&mut self) -> &mut KeybindManager<Action>;

//     fn handle_event(&mut self, event: Event) -> Option<ParentAction> {
//         match event {
//             Event::Key(event) => {
//                 tracing::debug!("{:#?}", event);

//                 match self.keybinding_manager().handle_event(event) {
//                     HandleEvent::Action(action) => {
//                         let action = self.apply_action(action);
//                         action
//                     }
//                     HandleEvent::Registered => None,
//                     HandleEvent::Cancel => None,
//                 }
//             }
//             Event::Mouse(_event) => None,
//             _ => None,
//         }
//     }
// }
