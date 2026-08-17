use std::sync::Arc;

use crate::backend::{Backend, MailDataAttachment};

pub struct MailDisplayAttachment {
    pub downloaded: bool,
    pub name: String,
    pub content_type: String,
    pub size: String,
}

impl MailDisplayAttachment {
    pub const MAX_DISPLAY_LENGTH: u16 = 5;

    pub fn new(attachment: MailDataAttachment, backend: Arc<Backend>) -> Self {
        let size = {
            const KB: f64 = 1024.0;
            const MB: f64 = KB * 1024.0;
            const GB: f64 = MB * 1024.0;

            let size = attachment.size as f64;
            if size >= GB {
                format!("{:.1}G", size / GB)
            } else if size >= MB {
                format!("{:.1}M", size / MB)
            } else if size >= KB {
                format!("{:.1}K", size / KB)
            } else {
                format!("{}B", attachment.size)
            }
        };

        let downloaded = backend.get_attachment_path(&attachment).is_some();

        Self {
            name: attachment.name,
            content_type: attachment.content_type,
            size,
            downloaded,
        }
    }
}
