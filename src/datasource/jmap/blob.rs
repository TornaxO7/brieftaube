use crate::{
    datasource::{BlobRemote, jmap::JmapAccount},
    types::BlobId,
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl BlobRemote for JmapAccount {
    async fn fetch_blob(&self, id: BlobId) -> Result<Vec<u8>> {
        self.client
            .download(id.as_str())
            .await
            .map_err(|err| err.into())
    }
}
