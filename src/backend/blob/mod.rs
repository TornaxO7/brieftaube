pub mod types;

use crate::backend::blob::types::BlobId;

use super::Backend;

impl Backend {
    pub async fn download_blob(&self, id: &BlobId) -> Result<(), jmap_client::Error> {
        let bytes = self.client.download(&id.0).await?;
    }
}
