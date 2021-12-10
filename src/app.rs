use crossterm::event::KeyCode;
use itertools::Itertools;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::client::{
    audit::{AuditInfo, RelayedInfo},
    channel::{ChannelInfo, ChannelState},
    node::{NetworkNode, NodeInfo},
    Client,
};

pub type AppMutex = Arc<Mutex<App>>;

pub struct App {
    pub client: Client,
    pub db: sled::Db,

    pub tabs: Vec<String>,
    pub tab_index: usize,

    pub errors: Vec<String>,

    pub node_info: NodeInfo,
    pub active_chans: usize,
    pub pending_chans: usize,
    pub sleeping_chans: usize,

    pub active_sats: u64,
    pub pending_sats: u64,
    pub sleeping_sats: u64,

    pub relayed_count_mounth: u64,
    pub relayed_count_day: u64,
    pub relayed_mounth: u64,
    pub relayed_day: u64,

    pub fee_mounth: u64,
    pub fee_day: u64,
    pub return_rate: f64, // ARP per year

    pub screen_width: u16,
    pub relays_maximum_volume: u64,
    pub relays_maximum_count: u64,
    pub relays_amounts_line: Vec<u64>,
    pub relays_volumes_line: Vec<u64>,

    pub channels_stats: Vec<ChannelStats>,

    pub channels: Vec<ChannelInfo>,
    pub audit: AuditInfo,
    pub known_nodes: HashMap<String, NetworkNode>,
}

#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub node_id: String,
    pub chan_id: String,
    pub alias: String,
    pub local: u64,
    pub remote: u64,
    pub relays_amount: u64,
    pub relays_volume: u64,
    pub relays_fees: u64,
}

impl App {
    pub async fn new(client: Client, db: sled::Db) -> Result<App, Box<dyn Error>> {
        let node_info = client.get_info().await?;

        Ok(App {
            client,
            db,
            tabs: vec![
                "Dashboard".to_owned(),
                "Peers".to_owned(),
                "Onchain".to_owned(),
                "Routing".to_owned(),
            ],
            tab_index: 0,
            errors: vec![],
            node_info,
            active_chans: 0,
            pending_chans: 0,
            sleeping_chans: 0,
            active_sats: 0,
            pending_sats: 0,
            sleeping_sats: 0,
            relayed_count_mounth: 0,
            relayed_count_day: 0,
            relayed_mounth: 0,
            relayed_day: 0,
            fee_mounth: 0,
            fee_day: 0,
            return_rate: 0.0,
            screen_width: 80,
            relays_maximum_volume: 0,
            relays_maximum_count: 0,
            relays_amounts_line: vec![],
            relays_volumes_line: vec![],
            channels_stats: vec![],
            channels: vec![],
            audit: AuditInfo::default(),
            known_nodes: HashMap::new(),
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
        match k {
            KeyCode::Char('d') => self.tab_index = 0,
            KeyCode::Char('p') => self.tab_index = 1,
            KeyCode::Char('o') => self.tab_index = 2,
            KeyCode::Char('r') => self.tab_index = 3,
            _ => (),
        }
    }

    pub fn get_active_chans(&self) -> usize {
        self.channels
            .iter()
            .filter(|c| c.state == ChannelState::Normal)
            .count()
    }

    pub fn get_pending_chans(&self) -> usize {
        self.channels
            .iter()
            .filter(|c| {
                c.state == ChannelState::Closing
                    || c.state == ChannelState::Opening
                    || c.state == ChannelState::Syncing
            })
            .count()
    }

    pub fn get_sleeping_chans(&self) -> usize {
        self.channels
            .iter()
            .filter(|c| c.state == ChannelState::Offline)
            .count()
    }

    pub fn get_active_sats(&self) -> u64 {
        self.channels
            .iter()
            .filter(|c| c.state == ChannelState::Normal)
            .map(|c| c.data.commitments.local_commit.spec.to_local)
            .sum()
    }

    pub fn get_pending_sats(&self) -> u64 {
        self.channels
            .iter()
            .filter(|c| {
                c.state == ChannelState::Closing
                    || c.state == ChannelState::Opening
                    || c.state == ChannelState::Syncing
            })
            .map(|c| c.data.commitments.local_commit.spec.to_local)
            .sum()
    }

    pub fn get_sleeping_sats(&self) -> u64 {
        self.channels
            .iter()
            .filter(|c| c.state == ChannelState::Offline)
            .map(|c| c.data.commitments.local_commit.spec.to_local)
            .sum()
    }

    fn get_relayed(&self, interval: i64) -> u64 {
        let now = chrono::offset::Utc::now().timestamp();
        self.audit
            .relayed
            .iter()
            .filter(|s| s.timestamp / 1000 > (now - interval) as u64)
            .map(|s| s.amount_in)
            .sum()
    }

    pub fn get_relayed_mounth(&self) -> u64 {
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
            .filter(|s| s.timestamp / 1000 > (now - interval) as u64)
            .map(|_| 1)
            .sum()
    }

    pub fn get_relayed_count_mounth(&self) -> u64 {
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
            .filter(|s| s.timestamp / 1000 > (now - interval) as u64)
            .map(|s| s.amount_in - s.amount_out)
            .sum()
    }

    pub fn get_fee_mounth(&self) -> u64 {
        self.get_fee(30 * 24 * 3600)
    }

    pub fn get_fee_day(&self) -> u64 {
        self.get_fee(24 * 3600)
    }

    pub fn get_return_rate(&self) -> f64 {
        12.0 * 100.0 * (self.fee_mounth as f64) / (self.local_volume() as f64)
    }

    pub fn local_volume(&self) -> u64 {
        self.active_sats + self.pending_sats + self.sleeping_sats
    }

    pub fn relayed_percent(&self) -> f64 {
        100.0 * (self.relayed_mounth as f64) / (self.local_volume() as f64)
    }

    const LINE_PERIOD: u64 = 24 * 3600;
    const LINE_MARGINS: u64 = 2;

    pub fn get_relays_amounts_line(&mut self) -> (Vec<u64>, u64) {
        let now = chrono::offset::Utc::now().timestamp();
        let mut relays: Vec<u64> = self
            .audit
            .relayed
            .iter()
            .filter(|s| s.timestamp / 1000 > (now - App::LINE_PERIOD as i64) as u64)
            .map(|s| s.timestamp)
            .collect();
        relays.sort_by(|a, b| a.partial_cmp(&b).unwrap());

        let line_width = self.screen_width as u64 - App::LINE_MARGINS;
        let mut result = vec![0; line_width as usize + 1];
        let mut max_relay = 0;
        if !relays.is_empty() {
            let t0 = relays[0];
            let t1 = relays[relays.len() - 1];
            for t in relays.iter() {
                let i = (((t - t0) as f64) / ((t1 - t0) as f64) * (line_width as f64)) as usize;
                result[i] += 1;
            }

            max_relay = *result.iter().max().unwrap_or(&1);
            result = result
                .iter()
                .map(|a| (100.0 * (*a as f64) / (max_relay as f64)) as u64)
                .collect();
        }
        (result, max_relay)
    }

    pub fn get_relays_volumes_line(&mut self) -> (Vec<u64>, u64) {
        let now = chrono::offset::Utc::now().timestamp();
        let mut relays: Vec<(u64, u64)> = self
            .audit
            .relayed
            .iter()
            .filter(|s| s.timestamp / 1000 > (now - App::LINE_PERIOD as i64) as u64)
            .map(|s| (s.amount_in, s.timestamp))
            .collect();
        relays.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let line_width = self.screen_width as u64 - App::LINE_MARGINS;
        let mut result = vec![0; line_width as usize + 1];
        let mut max_relay = 0;
        if !relays.is_empty() {
            let t0 = relays[0].1;
            let t1 = relays[relays.len() - 1].1;
            for (amount, t) in relays.iter() {
                let i = (((t - t0) as f64) / ((t1 - t0) as f64) * (line_width as f64)) as usize;
                result[i] += amount;
            }

            max_relay = *result.iter().max().unwrap_or(&1);
            result = result
                .iter()
                .map(|a| (100.0 * (*a as f64) / (max_relay as f64)) as u64)
                .collect();
        }
        (result, max_relay)
    }

    pub async fn start_workers(mapp: AppMutex) {
        tokio::spawn({
            let mapp = mapp.clone();
            async move {
                loop {
                    let res = query_node_info(mapp.clone()).await;
                    match res {
                        Err(e) => {
                            let now = chrono::offset::Utc::now().timestamp();
                            let estr = format!("App worker failed at {} with: {}", now, e);
                            // println!("{}", estr);
                            let mut app = mapp.lock().unwrap();
                            app.errors.push(estr);
                        }
                        _ => {
                            // let mut app = mapp.lock().unwrap();
                            // let num_chans = app.channels.len();
                            // app.errors.push(format!("All is ok! Got channels: {}", num_chans))
                        }
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

    pub fn get_channels_stats(&self) -> Vec<ChannelStats> {
        self.channels
            .iter()
            .map(|c| self.get_channel_stats(c))
            .collect()
    }

    pub fn get_channel_stats(&self, chan: &ChannelInfo) -> ChannelStats {
        let now = chrono::offset::Utc::now().timestamp();
        let interval = 24 * 3600;
        let relays: Vec<&RelayedInfo> = self.audit
            .relayed
            .iter()
            .filter(|s| (s.from_channel_id == chan.channel_id || s.to_channel_id == chan.channel_id) && s.timestamp / 1000 > (now - interval) as u64)
            .collect();

        ChannelStats {
            node_id: chan.node_id.clone(),
            chan_id: chan.channel_id.clone(),
            alias: self
                .known_nodes
                .get(&chan.node_id)
                .map(|n| n.alias.clone())
                .unwrap_or_else(|| chan.node_id.clone()),
            local: chan.data.commitments.local_commit.spec.to_local,
            remote: chan.data.commitments.local_commit.spec.to_remote,
            relays_amount: relays.iter().map(|_| 1).sum(),
            relays_volume: relays.iter().map(|r| r.amount_in).sum(),
            relays_fees: relays.iter().map(|r| r.amount_in - r.amount_out).sum(),
        }
    }
}

pub async fn query_node_info(mapp: AppMutex) -> Result<(), super::client::Error> {
    let client = mapp.lock().unwrap().client.clone();
    let chan_info = client.get_channels().await?;
    let audit_info = client.get_audit().await?;

    let channel_nodes: Vec<&str> = chan_info.iter().map(|c| &c.node_id[..]).unique().collect();
    let nodes_info = client.get_nodes(&channel_nodes).await?;

    {
        let mut app = mapp.lock().unwrap();
        app.channels = chan_info;
        app.active_chans = app.get_active_chans();
        app.pending_chans = app.get_pending_chans();
        app.sleeping_chans = app.get_sleeping_chans();
        app.active_sats = app.get_active_sats();
        app.pending_sats = app.get_pending_sats();
        app.sleeping_sats = app.get_sleeping_sats();

        app.audit = audit_info;
        let (amounts, max_amounts) = app.get_relays_amounts_line();
        app.relays_amounts_line = amounts;
        app.relays_maximum_count = max_amounts;
        let (volumes, max_volume) = app.get_relays_volumes_line();
        app.relays_volumes_line = volumes;
        app.relays_maximum_volume = max_volume;

        app.relayed_mounth = app.get_relayed_mounth();
        app.relayed_day = app.get_relayed_day();
        app.relayed_count_mounth = app.get_relayed_count_mounth();
        app.relayed_count_day = app.get_relayed_count_day();

        app.fee_mounth = app.get_fee_mounth();
        app.fee_day = app.get_fee_day();
        app.return_rate = app.get_return_rate();

        app.known_nodes = nodes_info
            .iter()
            .map(|n| (n.node_id.clone(), n.clone()))
            .collect();
        app.channels_stats = app.get_channels_stats();
    }
    Ok(())
}
