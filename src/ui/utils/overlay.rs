use crate::ui::utils::palette::HandleEventResult;
use crossterm::event::KeyEvent;

pub trait WidgetOverlay<E> {
    fn handle_event(&mut self, event: KeyEvent) -> Option<HandleEventResult<E>>;
}
