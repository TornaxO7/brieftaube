use super::keybindmanager::{HandleEvent, KeybindManager};
use crossterm::event::Event;

pub trait LayerCore {
    fn handle_event(
        &mut self,
        event: Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<crate::Action>;
}

pub trait LayerOverlay: LayerCore {
    fn into_message(self) -> Option<String>;
}

pub trait LayerModel<Action>: LayerCore
where
    Action: Clone,
{
    #[must_use]
    fn apply_action(&mut self, action: Action) -> Option<crate::Action>;

    #[must_use]
    fn handle_overlay<O>(&mut self, overlay: O) -> Option<crate::Action>
    where
        O: LayerOverlay;

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action>;

    fn handle_event(
        &mut self,
        event: Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<crate::Action> {
        match event {
            Event::Key(event) => {
                tracing::debug!("{:#?}", event);

                statusbar.push_key_press(event);
                match self.keybinding_manager().handle_event(event) {
                    HandleEvent::Action(action) => {
                        let action = self.apply_action(action);
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
