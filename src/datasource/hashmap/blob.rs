use crate::{
    datasource::{BlobCache, hashmap::HashMapDataSource},
    types::BlobId,
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl BlobCache for HashMapDataSource {
    async fn get_blob(&self, id: &BlobId) -> Option<Vec<u8>> {
        self.blobs.get(&id).cloned()
    }

    async fn upsert_blob(&mut self, id: BlobId, blob: Vec<u8>) -> Result<()> {
        self.blobs.insert(id, blob);
        Ok(())
    }
}
