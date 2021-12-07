mod opts;

use clap::Parser;

use self::opts::Opts;

fn main() {
    let opts: Opts = Opts::parse();
    let db: sled::Db = sled::open(&opts.state).unwrap();
}
