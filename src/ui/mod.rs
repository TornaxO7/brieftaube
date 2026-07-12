pub mod composer;
pub mod log_viewer;
pub mod mail_viewer;
pub mod mailboxes;
pub mod root_mails;
pub mod thread_mails;
pub mod utils;

use crossterm::event::{Event, KeyEvent};
use utils::{input, keybindmanager::KeybindManager, palette};

pub type MailboxId = String;
pub type MailId = String;
pub type ThreadId = String;

pub trait ScreenState<A: Clone, P: Clone, I: Clone> {
    fn update(&mut self);

    fn apply_action(&mut self, action: A);

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action>;

    fn keybinding_manager(&mut self) -> &mut KeybindManager<A>;

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(event) => {
                if let Some(overlay) = self.overlay() {
                    if let Some(result) = overlay.handle_event(event) {
                        self.handle_overlay_result(result);
                    }
                    return;
                }

                if let Some(action) = self.keybinding_manager().handle_event(event) {
                    self.apply_action(action);
                }
            }
            Event::Mouse(_event) => {}
            _ => {}
        }
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<P, I>>;

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<P, I>);
}

pub enum ScreenOverlay<P: Clone, I: Clone> {
    Palette(palette::State<P>),
    Input(input::State<I>),
}

pub enum ScreenOverlayResult<P, I> {
    Palette(P),
    Input { value: String, typ: I },
    Cancel,
}

impl<P: Clone, I: Clone> ScreenOverlay<P, I> {
    pub fn handle_event(&mut self, event: KeyEvent) -> Option<ScreenOverlayResult<P, I>> {
        match self {
            Self::Palette(state) => state.handle_event(event),
            Self::Input(state) => state.handle_event(event),
        }
    }
}
