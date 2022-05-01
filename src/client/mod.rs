pub mod audit;
pub mod channel;
pub mod common;
pub mod hosted;
pub mod node;

use self::{
    audit::AuditInfo,
    channel::ChannelInfo,
    hosted::{FcInfo, HcInfo},
    node::{NetworkNode, NodeInfo},
};
use log::*;
use std::collections::{HashMap, HashSet};
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

/// Additional plugins of Eclair node that we know about
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodePlugin {
    /// https://github.com/engenegr/plugin-hosted-channels
    HostedChannels,
    /// https://github.com/standardsats/plugin-fiat-channels
    FiatChannels,
}

impl std::fmt::Display for NodePlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NodePlugin::HostedChannels => write!(f, "hosted channels"),
            NodePlugin::FiatChannels => write!(f, "fiat channels"),
        }
    }
}

impl NodePlugin {
    pub fn known() -> Vec<NodePlugin> {
        vec![NodePlugin::HostedChannels, NodePlugin::FiatChannels]
    }
}

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
        trace!("Response from info: {}", txt);
        #[cfg(feature = "trace-to-file")]
        {
            if log_enabled!(log::Level::Trace) {
                trace!("Response written to info_response.json");
                std::fs::write("info_response.json", &txt).expect("Unable to write file");
            }
        }
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
        #[cfg(feature = "trace-to-file")]
        {
            if log_enabled!(log::Level::Trace) {
                trace!("Response written to channels_response.json");
                std::fs::write("channels_response.json", &txt).expect("Unable to write file");
            }
        }
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
        #[cfg(feature = "trace-to-file")]
        {
            if log_enabled!(log::Level::Trace) {
                trace!("Response written to audit_response.json");
                std::fs::write("audit_response.json", &txt).expect("Unable to write file");
            }
        }
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
        #[cfg(feature = "trace-to-file")]
        {
            if log_enabled!(log::Level::Trace) {
                trace!("Response written to nodes_response.json");
                std::fs::write("nodes_response.json", &txt).expect("Unable to write file");
            }
        }

        Ok(serde_json::from_str(&txt)?)
    }

    /// Probe a specific endpoint for plugin to test it availability on remote node
    pub async fn support_plugin(&self, plugin: NodePlugin) -> Result<bool> {
        let method = match plugin {
            NodePlugin::HostedChannels => format!("{}/{}", self.url, "hc-all"),
            NodePlugin::FiatChannels => format!("{}/{}", self.url, "fc-all"),
        };
        trace!("Checking if {plugin} is enabled at node");
        let res = self
            .client
            .post(method)
            .basic_auth("", Some(self.password.clone()))
            .send()
            .await?;
        match res.error_for_status() {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.status() == Some(reqwest::StatusCode::from_u16(404).unwrap()) {
                    Ok(false)
                } else {
                    Err(err)?
                }
            }
        }
    }

    /// Probe all known plugins and collect supported ones to set
    pub async fn get_supported_plugins(&self) -> Result<HashSet<NodePlugin>> {
        let mut res = HashSet::new();
        for plugin in NodePlugin::known() {
            let supported = self.support_plugin(plugin).await?;
            if supported {
                res.insert(plugin);
            }
        }
        Ok(res)
    }

    pub async fn get_fiat_channels(&self) -> Result<FcInfo> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "fc-all"))
                .basic_auth("", Some(self.password.clone()))
        };
        trace!("Requsting fc-all");
        let txt = builder().send().await?.error_for_status()?.text().await?;
        trace!("Response from fc-all: {}", txt);
        #[cfg(feature = "trace-to-file")]
        {
            if log_enabled!(log::Level::Trace) {
                trace!("Response written to fc_all_response.json");
                std::fs::write("fc_all_response.json", &txt).expect("Unable to write file");
            }
        }
        Ok(serde_json::from_str(&txt)?)
    }

    pub async fn get_hosted_channels(&self) -> Result<HcInfo> {
        let builder = || {
            self.client
                .post(format!("{}/{}", self.url, "hc-all"))
                .basic_auth("", Some(self.password.clone()))
        };
        trace!("Requsting hc-all");
        let txt = builder().send().await?.error_for_status()?.text().await?;
        trace!("Response from hc-all: {}", txt);
        #[cfg(feature = "trace-to-file")]
        {
            if log_enabled!(log::Level::Trace) {
                trace!("Response written to hc_all_response.json");
                std::fs::write("hc_all_response.json", &txt).expect("Unable to write file");
            }
        }
        Ok(serde_json::from_str(&txt)?)
    }
}
