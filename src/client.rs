use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Requesting server error: {0}")]
    ReqwestErr(#[from] reqwest::Error),
}

/// Alias for a `Result` with the error type `self::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Hold required information to query LN node
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
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub version: String,
    pub node_id: String,
    pub alias: String,
    pub color: String,
    pub features: NodeFeatures,
    pub chain_hash: String,
    pub network: NodeNetwork,
    pub block_height: u64,
    pub public_addresses: Vec<String>,
    pub instance_id: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeFeatures {
    pub activated: HashMap<String, FeatureStatus>,
    pub unknown: Vec<u32>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FeatureStatus {
    Optional,
    Mandatory,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum NodeNetwork {
    Testnet,
    Regtest,
    Mainnet,
}
