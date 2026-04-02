use std::process::ExitCode;

use clap::Parser;
use core_bin::{Args, init_tracing};
use core_lib::{GlobalState, LuaVM};

fn main() -> ExitCode {
    let args = Args::parse();
    init_tracing();

    if let Err(e) = run(args) {
        tracing::error!("Fatal daemon error: {:#}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run(args: Args) -> anyhow::Result<()> {
    tracing::info!("Core is starting");

    let lua = LuaVM::new()?;
    lua.load_user_config(&args.config_path)?;

    let _state = GlobalState::new(lua);

    Ok(())
}
