use crate::types::{CachedAttachment, CachedMail};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct AttachmentsTab {
    pub attachments: Vec<CachedAttachment>,
    pub state: TableState,
}

impl AttachmentsTab {
    pub fn get_selected_entry<'a>(&'a self) -> Option<&'a CachedAttachment> {
        let idx = self.state.selected()?;

        self.attachments.get(idx)
    }

    pub fn navigate_down(&mut self) {
        if self.attachments.is_empty() {
            return;
        }

        let idx = self.state.selected().unwrap();

        self.state
            .select(Some((idx + 1).min(self.attachments.len() - 1)));
    }

    pub fn navigate_up(&mut self) {
        self.state.select_previous();
    }

    pub fn navigate_to_top(&mut self) {
        self.state.select_first();
    }

    pub fn navigate_to_bottom(&mut self) {
        if self.attachments.is_empty() {
            return;
        }
        self.state.select(Some(self.attachments.len() - 1));
    }
}

impl From<&CachedMail> for AttachmentsTab {
    fn from(mail: &CachedMail) -> Self {
        let attachments = mail.attachments.clone().into_inner().unwrap();
        let initial_idx = (!attachments.is_empty()).then_some(0);

        Self {
            attachments,
            state: TableState::new().with_selected(initial_idx),
        }
    }
}
