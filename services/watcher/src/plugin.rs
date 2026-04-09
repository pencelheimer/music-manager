use std::{collections::HashSet, path::PathBuf, time::Duration};

use core_lib::messages::TrackCoordinator;
use kameo::prelude::ActorRef;
use notify::RecommendedWatcher;
use notify_debouncer_full::{Debouncer, RecommendedCache};

/// The WatcherService actor responsible for monitoring the file system.
///
/// It encapsulates the debouncer and routes filtered file events to a target actor.
pub struct WatcherService<C: TrackCoordinator> {
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
