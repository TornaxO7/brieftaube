use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use jmap_client::{client::Client, core::query::QueryResponse, mailbox::query::Filter};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget, Widget},
};
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct State {
    is_focussed: bool,

    list_state: ListState,

    rx: oneshot::Receiver<Result<QueryResponse, jmap_client::Error>>,
}

impl State {
    pub async fn new(client: Arc<Client>) -> Self {
        let (tx, rx) = oneshot::channel::<Result<QueryResponse, jmap_client::Error>>();

        tokio::spawn(async move {
            let mailboxes = client.mailbox_query(None::<Filter>, None::<Vec<_>>).await;
            tx.send(mailboxes).unwrap()
        });

        Self {
            rx,
            is_focussed: false,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('k') {
            return Some(super::Action::OpenCommandPalette);
        }

        match event.code {
            KeyCode::Char('q') => Some(super::Action::Quit),
            KeyCode::Char(':') => Some(super::Action::OpenCommandPalette),
            KeyCode::Up => Some(super::Action::SelectPreviousMailBox),
            KeyCode::Down => Some(super::Action::SelectNextMailBox),
            _ => None,
        }
    }

    pub fn set_focus(&mut self, focussed: bool) {
        self.is_focussed = focussed;
    }
}

/// API Public functions
impl State {
    pub fn select_next(&mut self) {
        self.list_state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.list_state.select_previous();
    }
}

impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = {
            let mut block = Block::bordered().title("Mailboxes");

            block = if self.is_focussed {
                block.border_style(Style::new().green())
            } else {
                block
            };

            block
        };

        StatefulWidget::render(
            List::new(["Mailbox 1", "Mailbox 2", "Mailbox 3"])
                .block(block)
                .highlight_style(Style::new().blue())
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut self.list_state,
        )
    }
}
