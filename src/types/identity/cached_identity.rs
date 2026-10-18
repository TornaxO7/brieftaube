use crate::types::IdentityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedIdentity {
    pub id: IdentityId,
    pub name: String,
    pub email: String,
}

impl From<jmap_client::identity::Identity> for CachedIdentity {
    fn from(identity: jmap_client::identity::Identity) -> Self {
        Self {
            id: identity.id.unwrap().into(),
            name: identity.name.unwrap_or_default(),
            email: identity.email.unwrap(),
        }
    }
}
