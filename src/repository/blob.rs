use crate::{
    repository::Repository,
    types::{AccountId, BlobId, UploadResponse},
};
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
    GetBlob {
        id: BlobId,
        tx: oneshot::Sender<color_eyre::Result<Vec<u8>>>,
    },
    UploadBlob {
        blob: Vec<u8>,
        tx: oneshot::Sender<color_eyre::Result<UploadResponse>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
        Self::Blob(cmd)
    }
}

#[derive(Default)]
pub struct Locks {}
impl Repository {
    pub async fn get_blob(&self, account_id: AccountId, id: BlobId) -> color_eyre::Result<Vec<u8>> {
        let opt_blob = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_blob(&id)
            .await;

        match opt_blob {
            Some(blob) => Ok(blob),
            None => {
                let blob = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_blob(id.clone())
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                cache_lock.upsert_blob(id.clone(), blob.clone()).await?;

                Ok(blob)
            }
        }
    }

    pub async fn upload_blob(
        &self,
        account_id: AccountId,
        blob: Vec<u8>,
    ) -> color_eyre::Result<UploadResponse> {
        let result = self
            .remote
            .get_remote_account(account_id.clone())
            .upload_blob(account_id.clone(), blob.clone())
            .await;

        if let Ok(result) = result.as_ref() {
            let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;
            cache_lock.upsert_blob(result.blob_id.clone(), blob).await?;
        }

        result
    }
}
