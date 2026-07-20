use super::MailboxDisplay;
use ratatui::{
    layout::Constraint,
    widgets::{Row, TableState},
};

const IS_SELECTED: &str = "";
const SORT_ORDER: &str = "Sort order";
const NAME: &str = "Name";
const UNREAD_MAILS: &str = "Unread mails";

pub struct ColumnData<'a> {
    pub mailboxes: Vec<MailboxDisplay>,
    pub state: Option<&'a mut TableState>,
}

impl<'a> ColumnData<'a> {
    pub fn header() -> Row<'a> {
        Row::new([IS_SELECTED, SORT_ORDER, NAME, UNREAD_MAILS])
    }

    pub fn widths() -> [Constraint; 4] {
        [
            Constraint::Length(1),
            Constraint::Length(SORT_ORDER.len() as u16),
            Constraint::Fill(1),
            Constraint::Length(UNREAD_MAILS.len() as u16),
        ]
    }
}
