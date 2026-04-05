use std::{collections::HashMap, process::ExitCode};

use clap::Parser as _;
use core_bin::{Args, Coordinator, init_tracing};
use core_lib::{GlobalState, LuaVM, ServicePlugin as _, db};
use kameo::actor::Spawn as _;
use service_watcher::WatcherService;

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

    let watcher = WatcherService::new(state, coordinator_ref)?;

    watcher.start_loop().await?;

    Ok(())
}
