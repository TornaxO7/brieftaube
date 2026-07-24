use crate::mailfs::widget::{ColumnData, column_data::RightColumn};

pub struct RenderData<'a> {
    pub left: Option<ColumnData<'a>>,
    pub center: Option<ColumnData<'a>>,
    pub right: RightColumn<'a>,
}
