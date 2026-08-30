use crate::mailfs::widget::{ColumnDisplay, column_data::RightColumn};

pub struct RenderData<'a> {
    pub mailbox_path: String,
    pub left: Option<ColumnDisplay<'a>>,
    pub center: Option<ColumnDisplay<'a>>,
    pub right: Option<RightColumn<'a>>,
}
