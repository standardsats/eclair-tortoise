pub mod audit;
pub mod channel;
pub mod node;

use log::*;
use self::{node::{NodeInfo, NetworkNode}, channel::ChannelInfo, audit::AuditInfo};
use std::collections::HashMap;
use thiserror::Error;
use std::time::Duration;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    ReqwestErr(#[from] reqwest::Error),
    #[error("Failed to decode: {0}")]
    DecodingErr(#[from] serde_json::Error),
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
                .timeout(Duration::from_secs(10))
        };
        trace!("Requsting getinfo");
        let txt = builder().send().await?.error_for_status()?.text().await?;
        trace!("Response from getinfo: {}", txt);
        Ok(serde_json::from_str(&txt)?)
    }

    pub async fn get_channels(&self) -> Result<Vec<ChannelInfo>> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "channels"))
                .basic_auth("", Some(self.password.clone()))
                .timeout(Duration::from_secs(10))
        };
        trace!("Requsting channels");
        let txt = builder().send().await?.error_for_status()?.text().await?;
        trace!("Response from channels: {}", txt);
        Ok(serde_json::from_str(&txt)?)
    }

    pub async fn get_audit(&self) -> Result<AuditInfo> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "audit"))
                .basic_auth("", Some(self.password.clone()))
                .timeout(Duration::from_secs(10))
        };
        trace!("Requsting audit");
        let txt = builder().send().await?.error_for_status()?.text().await?;
        trace!("Response from audit: {}", txt);
        Ok(serde_json::from_str(&txt)?)
    }

    /// Get information about given nodes
    pub async fn get_nodes(&self, ids: &[&str]) -> Result<Vec<NetworkNode>> {
        let mut params = HashMap::new();
        params.insert("nodeIds", ids.join(","));
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "nodes"))
                .form(&params)
                .basic_auth("", Some(self.password.clone()))
                .timeout(Duration::from_secs(10))
        };
        trace!("Requsting nodes");
        let txt = builder().send().await?.error_for_status()?.text().await?;
        trace!("Response from nodes: {}", txt);
        Ok(serde_json::from_str(&txt)?)
    }
}
