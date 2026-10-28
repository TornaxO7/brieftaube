use crate::{
    datasource::{BlobRemote, jmap::JmapAccount},
    types::{AccountId, BlobId, UploadResponse},
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

    async fn upload_blob(&self, account_id: AccountId, blob: Vec<u8>) -> Result<UploadResponse> {
        self.client
            .upload(Some(account_id.as_str()), blob, None)
            .await
            .map(UploadResponse::from)
            .map_err(|err| err.into())
    }
}
