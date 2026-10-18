use crate::{
    datasource::{IdentityCache, hashmap::HashMapDataSource, types::GetState},
    types::CachedIdentity,
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl IdentityCache for HashMapDataSource {
    async fn get_identity_state(&self) -> Option<&GetState> {
        self.identity_get_state.as_ref()
    }

    async fn set_identity_staten(&mut self, new_state: GetState) -> Result<()> {
        self.identity_get_state = Some(new_state);
        Ok(())
    }

    async fn get_all_identities(&self) -> Result<Option<Vec<CachedIdentity>>> {
        if self.identity_get_state.is_some() {
            let identities = self.identities.values().cloned().collect();
            Ok(Some(identities))
        } else {
            Ok(None)
        }
    }

    async fn upsert_identities(&mut self, identities: Vec<CachedIdentity>) -> Result<()> {
        for identity in identities {
            let id = identity.id.clone();
            self.identities.insert(id, identity);
        }

        Ok(())
    }
}
