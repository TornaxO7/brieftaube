use crate::mailfs::widget::{ColumnData, column_data::RightColumn};

pub struct RenderData<'a> {
    pub left: ColumnData<'a>,
    pub center: ColumnData<'a>,
    pub right: RightColumn<'a>,
}
