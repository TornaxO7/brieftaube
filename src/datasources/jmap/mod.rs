mod mail;

use std::collections::HashMap;

use crate::{
    datasources::{
        BaseDataSource,
        types::{GetState, QueryState},
    },
    types::MailboxId,
};
use jmap_client::client::{Client, Credentials};

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

pub struct Jmap {
    client: Client,

    mail_get_state: Option<GetState>,
    mailboxes_get_state: Option<GetState>,
    threads_get_state: Option<GetState>,
    root_mails_query_state: HashMap<MailboxId, QueryState>,
}

impl Jmap {
    pub fn new(client: Client) -> Self {
        Self {
            client,

            mail_get_state: None,
            mailboxes_get_state: None,
            threads_get_state: None,
            root_mails_query_state: HashMap::new(),
        }
    }

    pub async fn connect(desc: JmapDescriptor) -> Result<Self, Error> {
        let host =
            get_host_from_url(desc.server_url.as_str()).ok_or(Error::NoDoubleSlashFoundInUrl)?;

        let client = Client::new()
            .credentials(desc.credentials)
            .follow_redirects([host])
            .connect(&desc.server_url)
            .await?;

        Ok(Self::new(client))
    }
}

impl BaseDataSource for Jmap {
    type Error = jmap_client::Error;

    fn mail_get_state(&self) -> Option<GetState> {
        self.mail_get_state.clone()
    }

    fn mailboxes_get_state(&self) -> Option<GetState> {
        self.mailboxes_get_state.clone()
    }

    fn threads_get_state(&self) -> Option<GetState> {
        self.threads_get_state.clone()
    }

    fn root_mails_query_state(&self, id: &MailboxId) -> Option<QueryState> {
        self.root_mails_query_state.get(id).cloned()
    }
}

fn get_host_from_url<'a>(url: &'a str) -> Option<&'a str> {
    let pos = url.find("//")?;
    let stripped = url[pos + 2..].trim();
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
    }
}
