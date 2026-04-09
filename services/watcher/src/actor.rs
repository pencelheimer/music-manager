use std::{collections::HashSet, path::PathBuf, time::Duration};

use core_lib::{GlobalState, messages::TrackCoordinator};
use kameo::prelude::{Actor, ActorRef};
use tracing::info;

use crate::{WatcherService, config::Config, error::WatcherError};

/// Arguments required to initialize and spawn the WatcherService.
pub struct WatcherServiceArgs<C: TrackCoordinator> {
    /// Shared global application state.
    pub config: Config,
    /// The coordinator or handler actor that will receive the resulting events.
    pub target_actor: ActorRef<C>,
}

impl<C: TrackCoordinator> WatcherServiceArgs<C> {
    pub fn with_state(
        state: &GlobalState,
        target_actor: ActorRef<C>,
    ) -> Result<Self, WatcherError> {
        let config = Config::new(state)?;

        Ok(Self {
            config,
            target_actor,
        })
    }

    pub fn new(
        watch_dir: PathBuf,
        allowed_extensions: HashSet<String>,
        debounce_duration: Duration,
        target_actor: ActorRef<C>,
    ) -> Self {
        let config = Config {
            watch_dir,
            allowed_extensions,
            debounce_duration,
        };

        Self {
            config,
            target_actor,
        }
    }
}

impl<C: TrackCoordinator> Actor for WatcherService<C> {
    type Args = WatcherServiceArgs<C>;
    type Error = WatcherError;

    /// Initializes the actor, configures Lua settings, and starts the OS file watcher.
    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        info!("Starting FS Watcher on {:?}", args.config.watch_dir);

        let debouncer = Self::init_debouncer(&args.config, actor_ref)?;

        Ok(Self {
            watch_dir: args.config.watch_dir,
            target_actor: args.target_actor,
            allowed_extensions: args.config.allowed_extensions,
            debounce_duration: args.config.debounce_duration,
            debouncer,
        })
    }
}
