use crate::mailfs::widget::ColumnData;

pub struct RenderData<'a> {
    pub left: Option<ColumnData<'a>>,
    pub center: ColumnData<'a>,
    pub right: Option<ColumnData<'a>>,
}
