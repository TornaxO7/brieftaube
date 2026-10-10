use jmap_client::email::EmailBodyPart;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CachedAttachment {
    pub name: String,
    pub content_type: String,
    pub size: usize,
    pub blob_id: String,
}

impl From<&EmailBodyPart> for CachedAttachment {
    fn from(part: &EmailBodyPart) -> Self {
        Self {
            name: part.name().unwrap().to_owned(),
            content_type: part.content_type().unwrap().to_owned(),
            size: part.size().unwrap(),
            blob_id: part.blob_id().unwrap().to_owned(),
        }
    }
}
