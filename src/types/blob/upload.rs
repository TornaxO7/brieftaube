use crate::types::BlobId;

#[derive(Debug, Clone)]
pub struct UploadResponse {
    pub blob_id: BlobId,
    pub ty: String,
    pub size: usize,
}

impl From<jmap_client::blob::upload::UploadResponse> for UploadResponse {
    fn from(upload: jmap_client::blob::upload::UploadResponse) -> Self {
        Self {
            blob_id: upload.blob_id().into(),
            ty: upload.content_type().to_string(),
            size: upload.size(),
        }
    }
}
