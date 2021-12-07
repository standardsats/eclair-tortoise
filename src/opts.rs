use clap::{crate_version, Parser};

#[derive(Parser)]
#[clap(version=crate_version!(), author="NCrashed <ncrashed@protonmail.com>")]
pub struct Opts {
    /// The host name of the lightning node that we monitor.
    #[clap(short, long, default_value="127.0.0.1")]
    pub host: String,
    /// The port of the lightning node that we monitor. It is the API port, not the
    /// default protocol 9735 port.
    #[clap(short, long, default_value="8080")]
    pub port: u16,

    /// The password of API for the lightning node. Note that you SHOULD always use
    /// the option to pass it via the environment variable, not directly via the CLI argument.
    #[clap(long, env="ECLAIR_TORTOISE_API_PASSWORD", hide_env_values=true)]
    pub password: String,

    /// Path to the local state database directory. Require read-write access.
    #[clap(short, long, default_value="./tortoise.db")]
    pub state: String,
}