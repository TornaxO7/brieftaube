// pub mod composer;
// pub mod log_viewer;
pub mod mailfs;
pub mod palette;
// pub mod prompt;
// pub mod reader;
pub mod statusbar;
mod utils;

use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};

pub trait LayerCore<ParentAction = crate::Action> {
    fn handle_event(&mut self, event: Event) -> Option<ParentAction>;

    fn is_overlay(&self) -> bool {
        false
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect);
}

pub trait LayerOverlay: LayerCore {
    fn into_message(self) -> Option<String>;
}

pub trait LayerState<Action, ParentAction = crate::Action>: LayerCore<ParentAction> {
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
