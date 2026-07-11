pub mod keybindmanager;
pub mod palette;

use crossterm::event::Event;
use keybindmanager::KeybindManager;

pub type MailboxId = String;
pub type MailId = String;
pub type ThreadId = String;

pub trait ScreenState<A: Clone, PE: Clone>: ScreenPalette<PE> {
    fn update(&mut self);

    fn apply_action(&mut self, action: A);

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action>;

    fn keybinding_manager(&mut self) -> &mut KeybindManager<A>;

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(event) => {
                if let Some(p) = self.palette() {
                    if let Some(result) = p.handle_event(event) {
                        self.handle_palette_result(result);
                        return;
                    }
                }

                if let Some(action) = self.keybinding_manager().handle_event(event) {
                    self.apply_action(action);
                }
            }
            Event::Mouse(_event) => {}
            _ => {}
        }
    }
}

pub trait ScreenPalette<E: Clone> {
    fn palette(&mut self) -> Option<&mut palette::State<E>>;

    fn handle_palette_result(&mut self, result: palette::HandleEventResult<E>);
}
