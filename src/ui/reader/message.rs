use crossterm::event::Event;

pub enum Message {
    UserAction(super::UserAction),
    Event(Event),
    SelectedPaletteEntry(String),
}
