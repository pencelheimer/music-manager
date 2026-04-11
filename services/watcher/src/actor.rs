use std::{collections::HashSet, path::PathBuf, time::Duration};

use async_trait::async_trait;
use core_lib::{GlobalState, ServicePlugin, messages::CoordinatorForService};
use kameo::prelude::{Actor, ActorRef};
use notify::RecommendedWatcher;
use notify_debouncer_full::{Debouncer, RecommendedCache};
use tracing::info;

use crate::{config::Config, error::WatcherError};

/// The WatcherService actor responsible for monitoring the file system.
///
/// It encapsulates the debouncer and routes filtered file events to a target actor.
pub struct WatcherService<C: CoordinatorForService> {
    /// The directory being monitored.
    pub watch_dir: PathBuf,
    /// The actor where processed events (ReadyToProcess/Removed) are sent.
    pub target_actor: ActorRef<C>,
    /// Set of file extensions that this watcher cares about.
    pub allowed_extensions: HashSet<String>,
    /// Time to wait for file system stability before triggering events.
    pub debounce_duration: Duration,

    /// The underlying debouncer that must remain in scope for the duration of the actor's life.
    pub debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

/// Arguments required to initialize and spawn the WatcherService.
pub struct WatcherServiceArgs<C: CoordinatorForService> {
    /// Config of the actor.
    pub config: Config,
    /// The coordinator or handler actor that will receive the resulting events.
    pub target_actor: ActorRef<C>,
}

impl<C: CoordinatorForService> WatcherServiceArgs<C> {
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

impl<C: CoordinatorForService> Actor for WatcherService<C> {
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

#[async_trait]
impl<C: CoordinatorForService> ServicePlugin for WatcherService<C> {
    fn plugin_name() -> &'static str {
        "fs-watcher"
    }
}
