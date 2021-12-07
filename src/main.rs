mod app;
mod client;
mod opts;
mod ui;

use clap::Parser;
use std::error::Error;
use std::sync::{Arc, Mutex};

use self::app::App;
use self::client::Client;
use self::opts::Opts;
use self::ui::run_ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let db: sled::Db = sled::open(&opts.state)?;
    let client: Client = Client::new(&opts.url, &opts.password);

    let app = Arc::new(Mutex::new(App::new(client, db).await?));
    App::start_workers(app.clone()).await;
    run_ui(app)?;
    // loop {
    //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // }
    Ok(())
}
