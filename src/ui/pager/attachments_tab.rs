use crate::types::MailDataAttachment;
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct AttachmentsTab {
    pub attachments: color_eyre::Result<Vec<MailDataAttachment>>,
    pub state: TableState,
}

impl AttachmentsTab {
    pub fn new(attachments: color_eyre::Result<Vec<MailDataAttachment>>) -> Self {
        let initial_idx = match &attachments {
            Ok(attachments) => {
                if attachments.is_empty() {
                    None
                } else {
                    Some(0)
                }
            }
            Err(_) => None,
        };

        Self {
            attachments,
            state: TableState::new().with_selected(initial_idx),
        }
    }

    pub fn get_selected_entry<'a>(&'a self) -> Option<&'a MailDataAttachment> {
        let idx = self.state.selected()?;

        self.attachments
            .as_ref()
            .ok()
            .map(|attachments| &attachments[idx])
    }

    pub fn navigate_down(&mut self) {
        let Ok(attachments) = &self.attachments else {
            return;
        };

        let Some(idx) = self.state.selected() else {
            return;
        };

        if !attachments.is_empty() {
            self.state
                .select(Some((idx + 1).min(attachments.len() - 1)));
        }
    }

    pub fn navigate_up(&mut self) {
        if self.attachments.is_ok() {
            self.state.select_previous();
        }
    }

    pub fn navigate_to_top(&mut self) {
        if self.attachments.is_ok() {
            self.state.select_first();
        }
    }

    pub fn navigate_to_bottom(&mut self) {
        let Ok(attachments) = &self.attachments else {
            return;
        };

        if !attachments.is_empty() {
            self.state.select(Some(attachments.len() - 1));
        }
    }
}
