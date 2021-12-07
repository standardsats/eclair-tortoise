mod app;
mod client;
mod opts;
mod ui;

use clap::Parser;
use std::error::Error;

use self::app::App;
use self::client::Client;
use self::opts::Opts;
use self::ui::run_ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let db: sled::Db = sled::open(&opts.state)?;
    let client: Client = Client::new(&opts.url, &opts.password);

    let app = App::new(client, db).await?;
    run_ui(app)?;

    Ok(())
}
