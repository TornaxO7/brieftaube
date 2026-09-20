mod mail;
mod mailbox;
mod root_mails;
mod thread;

use crate::{
    datasource::{RemoteAccount, RemoteSession},
    types::{AccountData, AccountId},
};
use jmap_client::{
    client::{Client, Credentials},
    core::request::Request,
};
use std::sync::Arc;
use tracing::debug;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Client(#[from] jmap_client::Error),

    #[error("Couldn't find '//' within the url. It should be something like 'http://your.domain'.")]
    NoDoubleSlashFoundInUrl,
}

#[derive(Debug)]
pub struct JmapDescriptor {
    pub credentials: Credentials,
    pub server_url: String,
}

pub struct JmapSession {
    client: Arc<Client>,
}

impl JmapSession {
    pub async fn connect(desc: JmapDescriptor) -> Result<Self, Error> {
        let host =
            get_host_from_url(desc.server_url.as_str()).ok_or(Error::NoDoubleSlashFoundInUrl)?;

        debug!("Parsed host: {:?}", host);

        let client = Client::new()
            .credentials(desc.credentials)
            .follow_redirects([host])
            .connect(&desc.server_url)
            .await?;

        Ok(Self {
            client: Arc::new(client),
        })
    }
}

impl RemoteSession for JmapSession {
    fn get_accounts(&self) -> Vec<crate::types::AccountData> {
        let mut accounts = Vec::new();
        let session = self.client.session();

        for account_id in session.accounts() {
            let data = session.account(account_id).unwrap();

            if data.capability(jmap_client::URI::Mail.as_ref()).is_some() {
                accounts.push(AccountData {
                    id: account_id.clone().into(),
                    name: data.name().to_string(),
                });
            }
        }

        accounts
    }

    fn get_remote_account(&self, account_id: AccountId) -> Box<dyn RemoteAccount> {
        Box::new(JmapAccount {
            client: self.client.clone(),
            account_id,
        }) as Box<dyn RemoteAccount>
    }
}

pub struct JmapAccount {
    client: Arc<Client>,
    account_id: AccountId,
}

impl JmapAccount {
    pub fn build_request(&self) -> Request<'_> {
        self.client.build().account_id(self.account_id.0.clone())
    }
}

impl RemoteAccount for JmapAccount {}

fn get_host_from_url<'a>(url: &'a str) -> Option<&'a str> {
    let double_slash_pos = url.find("//")?;
    let port_pos = url[double_slash_pos..].find(":");

    let stripped = match port_pos {
        Some(port_pos) => url[double_slash_pos + 2..double_slash_pos + port_pos].trim(),
        None => url[double_slash_pos + 2..].trim(),
    };

    Some(stripped)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod host_from_url {
        use super::*;

        #[test]
        fn http() {
            let url = "http://test.domain";
            assert_eq!(get_host_from_url(url).unwrap(), "test.domain");
        }

        #[test]
        fn https() {
            let url = "https://test.domain";
            assert_eq!(get_host_from_url(url).unwrap(), "test.domain");
        }

        #[test]
        fn http_with_port() {
            let url = "http://test.domain:8080";
            assert_eq!(get_host_from_url(url).unwrap(), "test.domain");
        }

        #[test]
        fn https_with_port() {
            let url = "https://test.domain:8080";
            assert_eq!(get_host_from_url(url).unwrap(), "test.domain");
        }
    }
}
