use clap::{crate_version, Parser};

#[derive(Parser)]
#[clap(version=crate_version!(), author="NCrashed <ncrashed@protonmail.com>")]
pub struct Opts {
    /// The full url of the lightning node API that we monitor.
    #[clap(short, long, default_value = "http://127.0.0.1:8080")]
    pub url: String,

    /// The password of API for the lightning node. Note that you SHOULD always use
    /// the option to pass it via the environment variable, not directly via the CLI argument.
    #[clap(long, env = "ECLAIR_TORTOISE_API_PASSWORD", hide_env_values = true)]
    pub password: String,

    /// Path to the local state database directory. Require read-write access.
    #[clap(short, long, default_value = "./tortoise.db")]
    pub state: String,

    /// Logging level for putting messages into the log file.
    #[clap(short, long, default_value = "Warn", env = "RUST_LOG")]
    pub level: log::LevelFilter,

    /// Location of log file to write to
    #[clap(long, default_value = "./eclair-tortoise.log")]
    pub logfile: String,
}
