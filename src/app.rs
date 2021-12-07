use crossterm::event::KeyCode;
use std::error::Error;

use super::client::{Client, NodeInfo};

pub struct App<'a> {
    pub client: Client,
    pub db: sled::Db,

    pub tabs: Vec<&'a str>,
    pub tab_index: usize,

    pub node_info: NodeInfo,
}


impl<'a> App<'a> {
    pub async fn new(client: Client, db: sled::Db) -> Result<App<'a>, Box<dyn Error>> {
        let node_info = client.get_info().await?;

        Ok(App {
            client,
            db,
            tabs: vec!["Dashboard", "Peers", "Onchain", "Routing"],
            tab_index: 0,
            node_info,
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
}