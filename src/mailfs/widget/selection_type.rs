use crate::mailfs::state::SelectionType;

#[derive(Debug, Clone, Copy)]
pub enum DisplaySelectionType {
    Selected,
    Cut,
}

impl From<&SelectionType> for DisplaySelectionType {
    fn from(ty: &SelectionType) -> Self {
        match ty {
            SelectionType::Selected => Self::Selected,
            SelectionType::Cut => Self::Cut,
        }
    }
}
