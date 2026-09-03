mod mail;
mod mailbox;
mod root_mails;
mod thread;

use super::BaseDataSource;
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
}

impl Jmap {
    pub fn new(client: Client) -> Self {
        Self { client }
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
