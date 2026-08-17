use crate::backend::MailDataAttachment;

pub struct MailDisplayAttachment {
    pub name: String,
    pub content_type: String,
    pub size: String,
}

impl MailDisplayAttachment {
    pub const MAX_DISPLAY_LENGTH: u16 = 5;
}

impl From<MailDataAttachment> for MailDisplayAttachment {
    fn from(attachment: MailDataAttachment) -> Self {
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

        Self {
            name: attachment.name,
            content_type: attachment.content_type,
            size,
        }
    }
}
