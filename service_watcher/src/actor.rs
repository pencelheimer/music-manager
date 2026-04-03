use std::{collections::HashSet, convert::Infallible, path::PathBuf, time::Duration};

use core_lib::{GlobalState, ServicePlugin, messages::TrackEventHandler};
use kameo::{Actor, actor::ActorRef};
use notify_debouncer_full::new_debouncer;
use tracing::info;

use crate::{config::LuaWatcherExt, error::WatcherError};

#[derive(Debug)]
pub struct WatcherService<A: Actor + TrackEventHandler> {
    pub watch_dir: PathBuf,
    pub target_actor: ActorRef<A>,
    pub allowed_extensions: HashSet<String>,
}

impl<A: Actor + TrackEventHandler> WatcherService<A> {
    pub fn new(state: GlobalState, target_actor: ActorRef<A>) -> Result<Self, WatcherError> {
        let watch_dir = state.lua_vm.watch_dir()?;
        let allowed_extensions = state.lua_vm.allowed_extensions()?;

        Ok(Self {
            watch_dir,
            target_actor,
            allowed_extensions,
        })
    }
}

impl<A: Actor + TrackEventHandler> ServicePlugin<WatcherError> for WatcherService<A> {
    fn name() -> &'static str {
        "fs_watcher"
    }

    async fn start_loop(self) -> Result<Infallible, WatcherError> {
        let watch_dir = self.watch_dir.clone();

        info!("Starting FS Watcher on {:?}", watch_dir);

        // TODO(pencelheimer): maybe set debounce duration from the config
        let mut debouncer = new_debouncer(Duration::from_secs(2), None, self)?;
        debouncer.watch(&watch_dir, notify::RecursiveMode::Recursive)?;

        std::future::pending::<()>().await;

        unreachable!("FS Watcher stopped unexpectedly")
    }
}
