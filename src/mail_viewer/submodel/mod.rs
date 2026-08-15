pub mod attachments;
pub mod markdown;
pub mod metadata;
pub mod text;

use super::Action;

trait MailViewerSubModel {
    fn quit(&self) -> Option<Action> {
        Some(Action::Quit)
    }

    fn back(&self) -> Option<Action> {
        Some(Action::Back)
    }

    fn open_metadata_tab(&self) -> Option<Action> {
        Some(Action::OpenMetadataTab)
    }

    fn open_text_tab(&self) -> Option<Action> {
        Some(Action::OpenTextTab)
    }

    fn open_markdown_tab(&self) -> Option<Action> {
        Some(Action::OpenMarkdownTab)
    }

    fn open_attachments_tab(&self) -> Option<Action> {
        Some(Action::OpenAttachmentsTab)
    }

    fn open_next_tab(&self) -> Option<Action> {
        Some(Action::OpenNextTab)
    }

    fn open_previous_tab(&self) -> Option<Action> {
        Some(Action::OpenPreviousTab)
    }

    fn open_logs(&self) -> Option<Action> {
        Some(Action::OpenLogs)
    }
}
