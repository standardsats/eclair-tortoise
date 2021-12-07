use crossterm::event::KeyCode;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::client::{
    channel::{ChannelInfo, ChannelState},
    node::NodeInfo,
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

    pub channels: Vec<ChannelInfo>,
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
            channels: vec![],
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

    pub async fn start_workers(mapp: AppMutex) {
        tokio::spawn({
            let mapp = mapp.clone();
            async move {
                loop {
                    let res = query_node_info(mapp.clone()).await;
                    match res {
                        Err(e) => {
                            let estr = format!("App worker failed with: {}", e);
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
}

pub async fn query_node_info(mapp: AppMutex) -> Result<(), super::client::Error> {
    let client = mapp.lock().unwrap().client.clone();
    let chan_info = client.get_channels().await?;
    {
        let mut app = mapp.lock().unwrap();
        app.channels = chan_info;
        app.active_chans = app.get_active_chans();
        app.pending_chans = app.get_pending_chans();
        app.sleeping_chans = app.get_sleeping_chans();
    }
    Ok(())
}
