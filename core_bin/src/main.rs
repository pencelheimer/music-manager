use std::{collections::HashMap, process::ExitCode};

use clap::Parser as _;
use core_bin::{Args, Coordinator, init_tracing};
use core_lib::{GlobalState, LuaVM, db};
use kameo::actor::Spawn as _;
use tracing::info;
use watcher::{WatcherService, WatcherServiceArgs};

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    init_tracing();

    if let Err(e) = run(args).await {
        tracing::error!("Fatal daemon error: {:#}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run(args: Args) -> anyhow::Result<()> {
    tracing::info!("Core is starting");

    let lua = LuaVM::new()?;
    lua.load_user_config(&args.config_path)?;

    let db_pool = db::init(lua.db_url()?).await?;

    let state = GlobalState::new(lua, db_pool.clone());

    let plugins = HashMap::from([]);

    let coordinator = Coordinator::new(state.clone(), plugins)?;
    let coordinator_ref = Coordinator::spawn(coordinator);

    let watcher_args = WatcherServiceArgs::with_state(&state, coordinator_ref.clone())?;
    let watcher_ref = WatcherService::spawn(watcher_args);

    info!("Core is running");
    tokio::signal::ctrl_c().await?;

    info!("Shutdown signal received. Cleaning up...");

    watcher_ref.stop_gracefully().await?;
    coordinator_ref.stop_gracefully().await?;

    watcher_ref.wait_for_shutdown().await;
    coordinator_ref.wait_for_shutdown().await;

    info!("Core stopped safely");

    Ok(())
}
