pub mod input;
pub mod keybindmanager;
pub mod palette;

use crossterm::event::{Event, KeyEvent};
use keybindmanager::{HandleEvent, KeybindManager};

pub trait ScreenState<'a, A: Clone + std::fmt::Debug, P: Clone, I: Clone, R> {
    fn apply_action(&mut self, action: A) -> Option<crate::Action>;

    fn keybinding_manager(&mut self) -> &mut KeybindManager<A>;

    fn handle_event(
        &mut self,
        event: Event,
        statusbar: &mut crate::statusbar::State,
    ) -> Option<crate::Action> {
        match event {
            Event::Key(event) => {
                if let Some(overlay) = self.overlay() {
                    if let Some(result) = overlay.handle_event(event) {
                        self.handle_overlay_result(result);
                    }

                    return None;
                }

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

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<P, I>>;

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<P, I>);

    fn render_data(&'a mut self) -> R;
}

pub enum ScreenOverlay<P: Clone, I: Clone> {
    Palette(palette::State<P>),
    Input(input::State<I>),
}

impl<P: Clone, I: Clone> ScreenOverlay<P, I> {
    pub fn input<S: ToString>(desc: S, typ: I) -> Self {
        Self::Input(input::State::new(desc, typ))
    }
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
