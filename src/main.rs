mod app;
mod client;
mod opts;
mod ui;

#[macro_use(defer)]
extern crate scopeguard;

use clap::Parser;
use std::error::Error;
use std::sync::{Arc, Mutex};

use self::app::App;
use self::client::Client;
use self::opts::Opts;
use self::ui::run_ui;

use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let db: sled::Db = sled::open(&opts.state)?;
    let client: Client = Client::new(&opts.url, &opts.password);

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(opts.logfile)
        .unwrap();

    // Log to file with programmatically set level from CLI args
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(opts.level)))
                .build("logfile", Box::new(logfile)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config)?;

    let app = Arc::new(Mutex::new(App::new(client, db).await?));
    App::start_workers(app.clone()).await;
    run_ui(app)?;
    // loop {
    //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // }
    Ok(())
}
