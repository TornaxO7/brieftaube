use crate::mailfs::UserAction;

#[derive(Debug, Clone)]
pub enum PaletteValue {
    Action(UserAction),
}
