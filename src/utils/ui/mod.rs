pub mod input;
pub mod keybindmanager;

use crossterm::event::Event;
use keybindmanager::{HandleEvent, KeybindManager};

pub trait ScreenState<Action>
where
    Action: Clone,
{
    fn apply_user_action(&mut self, action: Action) -> Option<crate::Action>;

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action>;

    fn handle_event(
        &mut self,
        event: Event,
        statusbar: &mut crate::statusbar::State,
    ) -> Option<crate::Action> {
        match event {
            Event::Key(event) => {
                tracing::debug!("{:#?}", event);

                statusbar.push_key_press(event);
                match self.keybinding_manager().handle_event(event) {
                    HandleEvent::Action(action) => {
                        let action = self.apply_user_action(action);
                        statusbar.reset_key_press();
                        action
                    }
                    HandleEvent::Registered => None,
                    HandleEvent::Cancel => {
                        statusbar.reset_key_press();
                        None
                    }
                }
            }
            Event::Mouse(_event) => None,
            _ => None,
        }
    }
}
