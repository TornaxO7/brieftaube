use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MailViewer {
    pub default_tab: DefaultTab,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DefaultTab {
    Metadata,
    Text,
    Markdown,
    Attachments,
}
