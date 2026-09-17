use ratatui::widgets::TableState;

#[derive(Debug, Clone)]
pub struct Column<Ctx> {
    pub ctxs: Vec<Ctx>,
    pub state: TableState,
}

trait ColumnCtx {}
