
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::channel::{ChannelState, ChannelUpdate, ChannelAnnouncement, CommitSpec, TlvStream};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FcInfo {
    pub channels: HashMap<String, FiatChannel>
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HcInfo {
    pub channels: HashMap<String, HostedChannel>
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FiatChannel {
    pub state: ChannelState,
    pub data: FiatChanData,
    pub next_local_spec: CommitSpec,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HostedChannel {
    pub state: ChannelState,
    pub data: HostedChanData,
    pub next_local_spec: CommitSpec,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FiatChanData {
    pub commitments: Commits,
    pub channel_update: ChannelUpdate,
    pub local_errors: Vec<ChanError>,
    pub remote_errors: Option<Vec<ChanError>>,
    pub resize_proposal: Option<ResizeProposal>,
    pub override_proposal: Option<OverrideFiatProposal>,
    pub margin_proposal: Option<MarginProposal>,
    pub channel_announcement: Option<ChannelAnnouncement>,
    pub last_oracle_state: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HostedChanData {
    pub commitments: Commits,
    pub channel_update: ChannelUpdate,
    pub local_errors: Vec<ChanError>,
    pub remote_errors: Option<Vec<ChanError>>,
    pub resize_proposal: Option<ResizeProposal>,
    pub override_proposal: Option<OverrideHostedProposal>,
    pub margin_proposal: Option<MarginProposal>,
    pub channel_announcement: Option<ChannelAnnouncement>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Commits {
    pub local_node_id: String,
    pub remote_node_id: String,
    pub channel_id: String,
    pub local_spec: CommitSpec,
    pub origin_channels: HashMap<u64, OriginChannel>,
}

/// TODO: unclear codec format in scala
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OriginChannel {

}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChanError {
    pub error: LocalError,
    pub stamp: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalError {
    pub channel_id: String,
    pub data: String,
    pub tlv_stream: TlvStream,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResizeProposal {
    pub new_capacity: u64,
    pub client_sig: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverrideFiatProposal {
    pub block_day: u32,
    pub local_balance_msat: u64,
    pub local_updates: u32,
    pub remote_updates: u32,
    pub rate: u64,
    #[serde(rename = "localSigOfRemoteLCSS")]
    pub local_sig_of_remote_lcss: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverrideHostedProposal {
    pub block_day: u32,
    pub local_balance_msat: u64,
    pub local_updates: u32,
    pub remote_updates: u32,
    #[serde(rename = "localSigOfRemoteLCSS")]
    pub local_sig_of_remote_lcss: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginProposal {
    pub new_capacity: u64,
    pub new_rate: u64,
    pub client_sig: String,
}
