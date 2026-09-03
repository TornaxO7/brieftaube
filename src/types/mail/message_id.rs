#[derive(Debug, Clone)]
pub struct MessageId(pub String);

impl From<MessageId> for String {
    fn from(id: MessageId) -> Self {
        id.0
    }
}
