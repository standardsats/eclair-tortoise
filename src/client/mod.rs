pub mod audit;
pub mod channel;
pub mod node;

use thiserror::Error;
use self::{node::NodeInfo, channel::ChannelInfo, audit::AuditInfo};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    ReqwestErr(#[from] reqwest::Error),
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Hold required information to query LN node
#[derive(Clone)]
pub struct Client {
    url: String,
    password: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(url: &str, password: &str) -> Self {
        Client {
            url: url.to_owned(),
            password: password.to_owned(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_info(&self) -> Result<NodeInfo> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "getinfo"))
                .basic_auth("", Some(self.password.clone()))
        };
        // println!("{:?}", builder().send().await?.text().await?);
        Ok(builder().send().await?.error_for_status()?.json().await?)
    }

    pub async fn get_channels(&self) -> Result<Vec<ChannelInfo>> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "channels"))
                .basic_auth("", Some(self.password.clone()))
        };
        // println!("{:?}", builder().send().await?.text().await?);
        use std::io::prelude::*;
        let mut file = std::fs::File::create("log.txt").unwrap();
        let str = format!("{:?}", builder().send().await?.text().await?);
        file.write_all(str.as_bytes()).unwrap();
        Ok(builder().send().await?.error_for_status()?.json().await?)
    }

    pub async fn get_audit(&self) -> Result<AuditInfo> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "audit"))
                .basic_auth("", Some(self.password.clone()))
        };
        // println!("{:?}", builder().send().await?.text().await?);
        Ok(builder().send().await?.error_for_status()?.json().await?)
    }
}
