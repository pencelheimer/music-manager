use std::path::PathBuf;

use clap::Parser;

/// Music library manager daemon
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the `config.lua`
    #[arg(short, long = "config", default_value = "./config.lua")]
    pub config_path: PathBuf,
}
