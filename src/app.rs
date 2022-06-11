use crossterm::event::KeyCode;
use itertools::Itertools;
use log::*;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::client::{
    audit::{AuditInfo, RelayedInfo},
    channel::{ChannelInfo, ChannelState},
    hosted::{FcInfo, FiatChannel, HcInfo, HostedChannel},
    node::{NetworkNode, NodeInfo},
    Client, NodePlugin,
};

pub type AppMutex = Arc<Mutex<App>>;

pub struct App {
    pub client: Client,
    pub db: sled::Db,

    pub tabs: Vec<String>,
    pub tab_index: usize,

    pub errors: Vec<String>,

    pub supported: HashSet<NodePlugin>,
    pub stats_interval: i64,

    pub node_info: NodeInfo,
    pub active_chans: usize,
    pub pending_chans: usize,
    pub sleeping_chans: usize,

    pub active_sats: u64,
    pub pending_sats: u64,
    pub sleeping_sats: u64,

    pub relayed_count_month: u64,
    pub relayed_count_day: u64,
    pub relayed_month: u64,
    pub relayed_day: u64,

    pub fee_month: u64,
    pub fee_day: u64,
    pub return_rate: f64, // ARP per year

    pub screen_width: u16,
    pub relays_maximum_volume: u64,
    pub relays_maximum_count: u64,
    pub relays_amounts_line: Vec<u64>,
    pub relays_volumes_line: Vec<u64>,

    pub channels_stats: Vec<ChannelStats>,
    pub hosted_stats: Vec<ChannelStats>,
    pub fiat_stats: Vec<ChannelStats>,

    pub channels: Vec<ChannelInfo>,
    pub audit: AuditInfo,
    pub known_nodes: HashMap<String, NetworkNode>,
    pub hc_channels: HashMap<String, HostedChannel>,
    pub fc_channels: HashMap<String, FiatChannel>,

    // Dashboard screen
    pub search_focused: bool,
    pub search_line: String,
    pub channels_page: u64,

    // Channels screen
    pub chans_tab: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum ChannelType {
    Normal,
    Hosted,
    HostedFiat,
}

#[derive(Debug, Clone)]
pub enum ChannelExt {
    Normal,
    Hosted,
    HostedFiat(FiatChannelData),
}

impl ChannelExt {
    pub fn channel_type(&self) -> ChannelType {
        match self {
            ChannelExt::Normal => ChannelType::Normal,
            ChannelExt::Hosted => ChannelType::Hosted,
            ChannelExt::HostedFiat(_) => ChannelType::HostedFiat,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FiatChannelData {
    pub rate: u64,
    pub fiat_balance: f64,
}

impl FiatChannelData {
    pub fn reverse_rate(&self) -> f64 {
        100_000_000_000.0 / self.rate as f64
    }
}

#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub chan_state: ChannelState,
    pub node_id: String,
    pub chan_id: String,
    pub alias: String,
    pub local: u64,
    pub remote: u64,
    pub relays_amount: u64,
    pub relays_volume: u64,
    pub relays_fees: u64,
    pub info_id: usize,
    pub public: bool,
    pub channel_ext: ChannelExt,
}

impl ChannelStats {
    pub fn volume(&self) -> u64 {
        self.local + self.remote
    }

    pub fn is_normal_channel(&self) -> bool {
        self.channel_ext.channel_type() == ChannelType::Normal
    }

    pub fn fiat_balance(&self) -> f64 {
        match &self.channel_ext {
            ChannelExt::HostedFiat(data) => data.fiat_balance,
            _ => 0.0,
        }
    }

    pub fn rate(&self) -> u64 {
        match &self.channel_ext {
            ChannelExt::HostedFiat(data) => data.rate,
            _ => 0,
        }
    }

    pub fn reverse_rate(&self) -> f64 {
        match &self.channel_ext {
            ChannelExt::HostedFiat(data) => data.reverse_rate(),
            _ => 0.,
        }
    }
}

impl App {
    pub async fn new(client: Client, db: sled::Db) -> Result<App, Box<dyn Error>> {
        let node_info = client.get_info().await?;
        let supported = client.get_supported_plugins().await?;

        Ok(App {
            client,
            db,
            tabs: vec![
                "Dashboard".to_owned(),
                "Channels".to_owned(),
                "Peers".to_owned(),
                "Onchain".to_owned(),
                "Routing".to_owned(),
                "Hosted".to_owned(),
                "Fiat".to_owned(),
            ],
            tab_index: 0,
            errors: vec![],
            supported,
            stats_interval: 24 * 3600,
            node_info,
            active_chans: 0,
            pending_chans: 0,
            sleeping_chans: 0,
            active_sats: 0,
            pending_sats: 0,
            sleeping_sats: 0,
            relayed_count_month: 0,
            relayed_count_day: 0,
            relayed_month: 0,
            relayed_day: 0,
            fee_month: 0,
            fee_day: 0,
            return_rate: 0.0,
            screen_width: 80,
            relays_maximum_volume: 0,
            relays_maximum_count: 0,
            relays_amounts_line: vec![],
            relays_volumes_line: vec![],
            channels_stats: vec![],
            hosted_stats: vec![],
            fiat_stats: vec![],
            channels: vec![],
            audit: AuditInfo::default(),
            known_nodes: HashMap::new(),
            hc_channels: HashMap::new(),
            fc_channels: HashMap::new(),
            search_focused: false,
            search_line: "".to_owned(),
            channels_page: 0,
            chans_tab: 0,
        })
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.tabs.len() - 1;
        }
    }

    pub fn react_hotkey(&mut self, k: KeyCode) {
        if self.tab_index == 0 || self.tab_index == 5 || self.tab_index == 6 {
            match k {
                KeyCode::Up => {
                    self.channels_page = if self.channels_page == 0 {
                        0
                    } else {
                        self.channels_page - 1
                    }
                }
                KeyCode::Down => self.channels_page += 1,
                _ => (),
            }
        } else if self.tab_index == 1 {
            match k {
                KeyCode::Char('a') => self.chans_tab = 0,
                KeyCode::Char('e') => self.chans_tab = 1,
                KeyCode::Char('s') => self.chans_tab = 2,
                _ => (),
            }
        }

        match k {
            KeyCode::Char('d') => self.tab_index = 0,
            KeyCode::Char('c') => self.tab_index = 1,
            KeyCode::Char('p') => self.tab_index = 2,
            KeyCode::Char('o') => self.tab_index = 3,
            KeyCode::Char('r') => self.tab_index = 4,
            KeyCode::Char('h') => self.tab_index = 5,
            KeyCode::Char('f') => self.tab_index = 6,
            _ => (),
        }
    }

    pub fn get_active_chans(&self) -> usize {
        self.iterate_active_chans().count()
    }

    pub fn iterate_active_chans(&self) -> impl Iterator<Item = &ChannelInfo> {
        self.channels.iter().filter(|c| c.state.is_normal())
    }

    pub fn get_pending_chans(&self) -> usize {
        self.iterate_pending_chans().count()
    }

    pub fn iterate_pending_chans(&self) -> impl Iterator<Item = &ChannelInfo> {
        self.channels.iter().filter(|c| c.state.is_pending())
    }

    pub fn get_sleeping_chans(&self) -> usize {
        self.iterate_sleeping_chans().count()
    }

    pub fn iterate_sleeping_chans(&self) -> impl Iterator<Item = &ChannelInfo> {
        self.channels.iter().filter(|c| c.state.is_sleeping())
    }

    pub fn get_active_fiat_chans(&self) -> usize {
        self.iterate_active_fiat_chans().count()
    }

    pub fn get_suspended_fiat_chans(&self) -> usize {
        self.iterate_suspended_fiat_chans().count()
    }

    pub fn get_offline_fiat_chans(&self) -> usize {
        self.iterate_offline_fiat_chans().count()
    }

    pub fn iterate_active_fiat_chans(&self) -> impl Iterator<Item = (&String, &FiatChannel)> {
        self.fc_channels.iter().filter(|(_, c)| c.state.is_normal())
    }

    pub fn iterate_suspended_fiat_chans(&self) -> impl Iterator<Item = (&String, &FiatChannel)> {
        self.fc_channels
            .iter()
            .filter(|(_, c)| c.state.is_pending())
    }

    pub fn iterate_offline_fiat_chans(&self) -> impl Iterator<Item = (&String, &FiatChannel)> {
        self.fc_channels
            .iter()
            .filter(|(_, c)| c.state.is_sleeping())
    }

    pub fn get_active_sats(&self) -> u64 {
        self.channels
            .iter()
            .filter_map(|c| {
                if c.state == ChannelState::Normal {
                    c.data.as_ref()
                } else {
                    None
                }
            })
            .map(|c| c.commitments.local_commit.spec.to_local)
            .sum()
    }

    pub fn get_pending_sats(&self) -> u64 {
        self.channels
            .iter()
            .filter_map(|c| {
                if c.state.is_pending() {
                    c.data.as_ref()
                } else {
                    None
                }
            })
            .map(|c| c.commitments.local_commit.spec.to_local)
            .sum()
    }

    pub fn get_sleeping_sats(&self) -> u64 {
        self.channels
            .iter()
            .filter_map(|c| {
                if c.state.is_sleeping() {
                    c.data.as_ref()
                } else {
                    None
                }
            })
            .map(|c| c.commitments.local_commit.spec.to_local)
            .sum()
    }

    pub fn get_total_fiat_balance(&self) -> f64 {
        self.fiat_stats.iter().map(|s| s.fiat_balance()).sum()
    }

    pub fn get_fiat_balance_by<F: FnOnce(ChannelState) -> bool + Copy>(&self, f: F) -> f64 {
        self.fiat_stats
            .iter()
            .filter_map(|c| {
                if f(c.chan_state) {
                    Some(c.fiat_balance())
                } else {
                    None
                }
            })
            .sum()
    }

    fn get_relayed(&self, interval: i64) -> u64 {
        let now = chrono::offset::Utc::now().timestamp();
        self.audit
            .relayed
            .iter()
            .filter(|s| s.timestamp.unix > (now - interval) as u64)
            .map(|s| s.amount_in)
            .sum()
    }

    pub fn get_relayed_month(&self) -> u64 {
        self.get_relayed(30 * 24 * 3600)
    }

    pub fn get_relayed_day(&self) -> u64 {
        self.get_relayed(24 * 3600)
    }

    fn get_relayed_count(&self, interval: i64) -> u64 {
        let now = chrono::offset::Utc::now().timestamp();
        self.audit
            .relayed
            .iter()
            .filter(|s| s.timestamp.unix > (now - interval) as u64)
            .map(|_| 1)
            .sum()
    }

    pub fn get_relayed_count_month(&self) -> u64 {
        self.get_relayed_count(30 * 24 * 3600)
    }

    pub fn get_relayed_count_day(&self) -> u64 {
        self.get_relayed_count(24 * 3600)
    }

    fn get_fee(&self, interval: i64) -> u64 {
        let now = chrono::offset::Utc::now().timestamp();
        self.audit
            .relayed
            .iter()
            .filter(|s| s.timestamp.unix > (now - interval) as u64)
            .map(|s| s.amount_in - s.amount_out)
            .sum()
    }

    pub fn get_fee_month(&self) -> u64 {
        self.get_fee(30 * 24 * 3600)
    }

    pub fn get_fee_day(&self) -> u64 {
        self.get_fee(24 * 3600)
    }

    pub fn get_return_rate(&self) -> f64 {
        12.0 * 100.0 * (self.fee_month as f64) / (self.local_volume() as f64)
    }

    pub fn local_volume(&self) -> u64 {
        self.active_sats + self.pending_sats + self.sleeping_sats
    }

    pub fn relayed_percent(&self) -> f64 {
        100.0 * (self.relayed_month as f64) / (self.local_volume() as f64)
    }

    const LINE_PERIOD: u64 = 24 * 3600;
    const LINE_MARGINS: u64 = 2;

    pub fn get_relays_amounts_line(&mut self) -> (Vec<u64>, u64) {
        let now = chrono::offset::Utc::now().timestamp();
        let mut relays: Vec<u64> = self
            .audit
            .relayed
            .iter()
            .filter(|s| s.timestamp.unix > (now - App::LINE_PERIOD as i64) as u64)
            .map(|s| s.timestamp.unix)
            .collect();
        relays.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let line_width = self.screen_width as u64 - App::LINE_MARGINS - 1;
        let mut result = vec![0; line_width as usize + 1];
        let mut max_relay = 0;
        if !relays.is_empty() {
            let t0 = now as u64 - App::LINE_PERIOD;
            let t1 = now as u64;
            for t in relays.iter() {
                let i = (((t - t0) as f64) / ((t1 - t0) as f64) * (line_width as f64)) as usize;
                result[i] += 1;
            }

            if let Some(max) = result.iter().max() {
                max_relay = *max;
                if max_relay > 0 {
                    result = result
                        .iter()
                        .map(|a| (100.0 * (*a as f64) / (max_relay as f64)) as u64)
                        .collect();
                } else {
                    result = vec![];
                }
            } else {
                max_relay = 0;
                result = vec![];
            }
        }
        (result, max_relay)
    }

    pub fn get_relays_volumes_line(&mut self) -> (Vec<u64>, u64) {
        let now = chrono::offset::Utc::now().timestamp();
        let mut relays: Vec<(u64, u64)> = self
            .audit
            .relayed
            .iter()
            .filter(|s| s.timestamp.unix > (now - App::LINE_PERIOD as i64) as u64)
            .map(|s| (s.amount_in, s.timestamp.unix))
            .collect();
        relays.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let line_width = self.screen_width as u64 - App::LINE_MARGINS - 1;
        let mut result = vec![0; line_width as usize + 1];
        let mut max_relay = 0;
        if !relays.is_empty() {
            let t0 = now as u64 - App::LINE_PERIOD;
            let t1 = now as u64;
            for (amount, t) in relays.iter() {
                let i = (((t - t0) as f64) / ((t1 - t0) as f64) * (line_width as f64)) as usize;
                result[i] += amount;
            }

            if let Some(max) = result.iter().max() {
                max_relay = *max;
                if max_relay > 0 {
                    result = result
                        .iter()
                        .map(|a| (100.0 * (*a as f64) / (max_relay as f64)) as u64)
                        .collect();
                } else {
                    result = vec![];
                }
            } else {
                max_relay = 0;
                result = vec![];
            }
        }
        (result, max_relay)
    }

    pub async fn start_workers(mapp: AppMutex) {
        tokio::spawn({
            let mapp = mapp.clone();
            async move {
                loop {
                    let res = query_node_info(mapp.clone()).await;
                    if let Err(e) = res {
                        let now = chrono::offset::Utc::now().timestamp();
                        let estr = format!("App worker failed at {} with: {}", now, e);
                        error!("{}", estr);
                        let mut app = mapp.lock().unwrap();
                        app.errors.push(estr);
                    }
                    tokio::time::sleep(Duration::from_secs(20)).await;
                }
            }
        });
    }

    pub fn resize(&mut self, new_width: u16) {
        if self.screen_width != new_width {
            self.screen_width = new_width;
            let (amounts, max_amounts) = self.get_relays_amounts_line();
            self.relays_amounts_line = amounts;
            self.relays_maximum_count = max_amounts;
            let (volumes, max_volume) = self.get_relays_volumes_line();
            self.relays_volumes_line = volumes;
            self.relays_maximum_volume = max_volume;
        }
    }

    pub fn get_channels_stats(&self, interval: i64) -> Vec<ChannelStats> {
        self.channels
            .iter()
            .enumerate()
            .map(|(i, c)| self.get_channel_stats(i, interval, c))
            .collect()
    }

    pub fn get_hosted_stats(&self) -> Vec<ChannelStats> {
        self.hc_channels
            .iter()
            .enumerate()
            .map(|(i, (chanid, c))| self.get_hosted_channel_stats(i, chanid, c))
            .collect()
    }

    pub fn get_fiat_stats(&self) -> Vec<ChannelStats> {
        self.fc_channels
            .iter()
            .enumerate()
            .map(|(i, (chanid, c))| self.get_fiat_channel_stats(i, chanid, c))
            .collect()
    }

    pub fn get_channel_stats(&self, i: usize, interval: i64, chan: &ChannelInfo) -> ChannelStats {
        let now = chrono::offset::Utc::now().timestamp();
        let relays: Vec<&RelayedInfo> = self
            .audit
            .relayed
            .iter()
            .filter(|s| {
                (s.from_channel_id == chan.channel_id || s.to_channel_id == chan.channel_id)
                    && s.timestamp.unix > (now - interval) as u64
            })
            .collect();

        ChannelStats {
            chan_state: chan.state,
            node_id: chan.node_id.clone(),
            chan_id: chan.channel_id.clone(),
            alias: self
                .known_nodes
                .get(&chan.node_id)
                .map(|n| n.alias.clone())
                .unwrap_or_else(|| chan.node_id.clone()),
            local: chan
                .data
                .as_ref()
                .map_or(0, |c| c.commitments.local_commit.spec.to_local),
            remote: chan
                .data
                .as_ref()
                .map_or(0, |c| c.commitments.local_commit.spec.to_remote),
            relays_amount: relays.iter().map(|_| 1).sum(),
            relays_volume: relays.iter().map(|r| r.amount_in).sum(),
            relays_fees: relays.iter().map(|r| r.amount_in - r.amount_out).sum(),
            info_id: i,
            public: chan.data.as_ref().map_or(false, |c| {
                c.commitments
                    .channel_flags
                    .announce_channel
                    .unwrap_or(false)
            }),
            channel_ext: if chan.data.is_none() {
                ChannelExt::Hosted
            } else {
                ChannelExt::Normal
            },
        }
    }

    pub fn get_hosted_channel_stats(
        &self,
        i: usize,
        channel_id: &str,
        chan: &HostedChannel,
    ) -> ChannelStats {
        let now = chrono::offset::Utc::now().timestamp();
        let interval = 24 * 3600;
        let relays: Vec<&RelayedInfo> = self
            .audit
            .relayed
            .iter()
            .filter(|s| {
                (s.from_channel_id == channel_id || s.to_channel_id == channel_id)
                    && s.timestamp.unix > (now - interval) as u64
            })
            .collect();
        let node_id = &chan.data.commitments.remote_node_id;
        ChannelStats {
            chan_state: chan.state,
            node_id: node_id.to_owned(),
            chan_id: channel_id.to_owned(),
            alias: self
                .known_nodes
                .get(&chan.data.commitments.remote_node_id)
                .map(|n| n.alias.clone())
                .unwrap_or_else(|| node_id.clone()),
            local: chan.data.commitments.local_spec.to_local,
            remote: chan.data.commitments.local_spec.to_remote,
            relays_amount: relays.iter().map(|_| 1).sum(),
            relays_volume: relays.iter().map(|r| r.amount_in).sum(),
            relays_fees: relays.iter().map(|r| r.amount_in - r.amount_out).sum(),
            public: false,
            info_id: i,
            channel_ext: ChannelExt::Hosted,
        }
    }

    pub fn get_fiat_channel_stats(
        &self,
        i: usize,
        channel_id: &str,
        chan: &FiatChannel,
    ) -> ChannelStats {
        let now = chrono::offset::Utc::now().timestamp();
        let interval = 24 * 3600;
        let relays: Vec<&RelayedInfo> = self
            .audit
            .relayed
            .iter()
            .filter(|s| {
                (s.from_channel_id == channel_id || s.to_channel_id == channel_id)
                    && s.timestamp.unix > (now - interval) as u64
            })
            .collect();
        let node_id = &chan.data.commitments.remote_node_id;
        let remote_balance = chan
            .data
            .commitments
            .last_cross_signed_state
            .remote_balance_msat;
        let rate = chan.data.commitments.last_cross_signed_state.rate;
        ChannelStats {
            chan_state: chan.state,
            node_id: node_id.to_owned(),
            chan_id: channel_id.to_owned(),
            alias: self
                .known_nodes
                .get(&chan.data.commitments.remote_node_id)
                .map(|n| n.alias.clone())
                .unwrap_or_else(|| node_id.clone()),
            local: chan
                .data
                .commitments
                .last_cross_signed_state
                .local_balance_msat,
            remote: remote_balance,
            relays_amount: relays.iter().map(|_| 1).sum(),
            relays_volume: relays.iter().map(|r| r.amount_in).sum(),
            relays_fees: relays.iter().map(|r| r.amount_in - r.amount_out).sum(),
            info_id: i,
            public: false,
            channel_ext: ChannelExt::HostedFiat(FiatChannelData {
                rate,
                fiat_balance: remote_balance as f64 / rate as f64,
            }),
        }
    }
}

pub async fn query_node_info(mapp: AppMutex) -> Result<(), super::client::Error> {
    trace!("Quering next node stats");
    let client = mapp.lock().unwrap().client.clone();
    trace!("Getting channels");
    let chan_info = client.get_channels().await?;
    trace!("Getting audit");
    let audit_info = client.get_audit().await?;

    trace!("Getting nodes for that channels");
    let channel_nodes: Vec<&str> = chan_info.iter().map(|c| &c.node_id[..]).unique().collect();
    let nodes_info = client.get_nodes(&channel_nodes).await?;

    let supported = mapp.lock().unwrap().supported.clone();
    trace!("Getting info about hosted channels");
    let hosted_chans: HcInfo = if supported.contains(&NodePlugin::HostedChannels) {
        client.get_hosted_channels().await?
    } else {
        HcInfo {
            channels: HashMap::new(),
        }
    };
    trace!("Getting info about fiat channels");
    let fiat_chans: FcInfo = if supported.contains(&NodePlugin::FiatChannels) {
        client.get_fiat_channels().await?
    } else {
        FcInfo {
            channels: HashMap::new(),
        }
    };

    {
        trace!("Start calculation");
        let mut app = mapp.lock().unwrap();

        app.channels = chan_info;
        app.hc_channels = hosted_chans.channels;
        app.fc_channels = fiat_chans.channels;
        trace!("Calculating channels activity");
        app.active_chans = app.get_active_chans();
        app.pending_chans = app.get_pending_chans();
        app.sleeping_chans = app.get_sleeping_chans();
        app.active_sats = app.get_active_sats();
        app.pending_sats = app.get_pending_sats();
        app.sleeping_sats = app.get_sleeping_sats();

        trace!("Calculating relays amounts");
        app.audit = audit_info;
        let (amounts, max_amounts) = app.get_relays_amounts_line();
        app.relays_amounts_line = amounts;
        app.relays_maximum_count = max_amounts;
        trace!("Calculating relays volumes");
        let (volumes, max_volume) = app.get_relays_volumes_line();
        app.relays_volumes_line = volumes;
        app.relays_maximum_volume = max_volume;

        trace!("Calculating relays month");
        app.relayed_month = app.get_relayed_month();
        trace!("Calculating relays day");
        app.relayed_day = app.get_relayed_day();
        trace!("Calculating relays count month");
        app.relayed_count_month = app.get_relayed_count_month();
        trace!("Calculating relays count day");
        app.relayed_count_day = app.get_relayed_count_day();

        trace!("Calculating fees");
        app.fee_month = app.get_fee_month();
        app.fee_day = app.get_fee_day();
        trace!("Calculating return rate");
        app.return_rate = app.get_return_rate();

        trace!("Getting map of known nodes");
        app.known_nodes = nodes_info
            .iter()
            .map(|n| (n.node_id.clone(), n.clone()))
            .collect();
        trace!("Calculation of channels stats");
        app.channels_stats = app.get_channels_stats(app.stats_interval);
        app.hosted_stats = app.get_hosted_stats();
        app.fiat_stats = app.get_fiat_stats();
        debug!("Fiat channels count {}", app.fiat_stats.len());
    }
    trace!("Updating is done");
    Ok(())
}
