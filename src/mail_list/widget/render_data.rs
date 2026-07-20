use crate::backend::mails::types::{MailDisplay, MailId};
use ratatui::widgets::TableState;
use std::collections::HashSet;

pub struct RenderData<'a> {
    pub rows: Vec<MailDisplay>,
    pub table_state: &'a mut TableState,
    pub selected: &'a HashSet<MailId>,
}
