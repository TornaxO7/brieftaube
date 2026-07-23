use crate::mailfs::Action;

#[derive(Debug, Clone)]
pub enum PaletteValue {
    Action(Action),
}
