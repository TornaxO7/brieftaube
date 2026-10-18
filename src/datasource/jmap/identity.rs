use crate::{
    datasource::{IdentityRemote, jmap::JmapAccount, types::remote},
    types::CachedIdentity,
};
use async_trait::async_trait;
use color_eyre::Result;
use jmap_client::identity::Property;

#[async_trait]
impl IdentityRemote for JmapAccount {
    async fn fetch_identities_all(&self) -> Result<remote::GetOneResult<Vec<CachedIdentity>>> {
        let mut response = {
            let mut request = self.build_request();

            request
                .get_identity()
                .properties([Property::Id, Property::Name, Property::Email]);

            request.send_get_identity().await?
        };

        Ok(remote::GetOneResult {
            value: response
                .take_list()
                .into_iter()
                .map(CachedIdentity::from)
                .collect(),
            state: response.take_state().into(),
        })
    }
}
