use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditInfo {
    pub sent: Vec<SentInfo>,
    pub received: Vec<ReceivedInfo>,
    pub relayed: Vec<RelayedInfo>,
}

impl Default for AuditInfo {
    fn default() -> AuditInfo {
        AuditInfo {
            sent: vec![],
            received: vec![],
            relayed: vec![],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SentInfo {
    pub _type: String,
    pub id: String,
    pub payment_hash: String,
    pub payment_preimage: String,
    pub recipient_amount: u64,
    pub recipient_node_id: String,
    pub parts: Vec<SentPart>
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceivedInfo {
    pub _type: String,
    pub payment_hash: String,
    pub parts: Vec<ReceivedPart>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelayedInfo {
    pub _type: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub payment_hash: String,
    pub from_channel_id: String,
    pub to_channel_id: String,
    pub timestamp: Timestamp,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SentPart {
    pub id: String,
    pub amount: u64,
    pub fees_paid: u64,
    pub to_channel_id: String,
    pub timestamp: Timestamp,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceivedPart {
    pub amount: u64,
    pub from_channel_id: String,
    pub timestamp: Timestamp,
}