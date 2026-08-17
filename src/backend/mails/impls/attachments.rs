use crate::backend::{Backend, MailDataAttachment};
use std::{fs::File, io::Write, os::unix::fs::PermissionsExt, path::PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum AttachmentDownloadError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Jmap(#[from] jmap_client::Error),
}

impl Backend {
    pub fn get_attachment_path(&self, attachment: &MailDataAttachment) -> Option<PathBuf> {
        let path = get_attachment_path(attachment);

        if path.exists() { Some(path) } else { None }
    }

    pub async fn download_attachment(
        &self,
        attachment: &MailDataAttachment,
    ) -> Result<(), AttachmentDownloadError> {
        let path = get_attachment_path(attachment);
        let data = self.client.download(&attachment.blob_id).await?;

        let mut file = File::create(path)?;
        file.write_all(&data)?;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms)?;

        Ok(())
    }
}

fn get_attachment_path(attachment: &MailDataAttachment) -> PathBuf {
    let file_name = format!("{}-{}", attachment.blob_id, attachment.name);

    crate::get_xdg().place_cache_file(file_name).unwrap()
}
