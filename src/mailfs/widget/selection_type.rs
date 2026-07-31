use crate::mailfs::state;

#[derive(Debug, Clone, Copy)]
pub enum SelectionType {
    Selected,
    Cut,
}

impl From<state::SelectionType> for SelectionType {
    fn from(ty: state::SelectionType) -> Self {
        match ty {
            state::SelectionType::Selected => Self::Selected,
            state::SelectionType::Cut => Self::Cut,
        }
    }
}
