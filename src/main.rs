mod client;
mod opts;

use clap::Parser;
use std::error::Error;

use self::client::Client;
use self::opts::Opts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let db: sled::Db = sled::open(&opts.state)?;
    let client: Client = Client::new(&opts.url, &opts.password);

    let node_info = client.get_info().await?;
    println!("Node information: {:?}", node_info);

    Ok(())
}
