
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetworkNode {
    pub signature: String,
    pub features: NodeFeatures,
    pub timestamp: u64,
    pub node_id: String,
    pub rgb_color: String,
    pub alias: String,
    pub addresses: Vec<String>,
}