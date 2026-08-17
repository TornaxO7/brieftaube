use super::Cache;
use crate::backend::{MailDataAttachment, blob::types::BlobId};
use std::path::PathBuf;

impl Cache {
    fn get_blob_path(id: &BlobId, name: &str) -> PathBuf {
        let file_name = format!("{}-{}", &id.0, name);

        crate::get_xdg().place_runtime_file(file_name).unwrap()
    }

    pub fn save_mail_attachment(&self, attachment: &MailDataAttachment) {}
}
