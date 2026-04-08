mod mock_coordinator;
mod mock_watcher;

pub use mock_coordinator::*;
pub use mock_watcher::*;

use std::{collections::HashSet, path::PathBuf, time::Duration};

use kameo::actor::{ActorRef, Spawn};
use once_cell::sync::Lazy;
use service_watcher::{WatcherService, WatcherServiceArgs};
use tokio::time::{sleep, timeout};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _};

static TRACING: Lazy<()> = Lazy::new(|| {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,core_lib=debug".into()))
        .with(fmt::layer().pretty())
        .init()
});

pub struct TestApp {
    pub coordinator: ActorRef<MockCoordinator>,
    pub watcher: ActorRef<WatcherService<MockCoordinator>>,
}

impl TestApp {
    pub async fn new<V: AsRef<[T]>, T: ToString>(watch_dir: PathBuf, exts: V) -> Self {
        Lazy::force(&TRACING);

        let allowed_extensions: HashSet<_> = exts.as_ref().iter().map(|s| s.to_string()).collect();

        let coordinator = MockCoordinator::spawn_default();
        let watcher_args = WatcherServiceArgs::new(
            watch_dir,
            allowed_extensions,
            Duration::from_millis(50),
            coordinator.clone(),
        );
        let watcher = WatcherService::spawn(watcher_args);

        watcher.ask(Ping).send().await.unwrap();

        Self {
            coordinator,
            watcher,
        }
    }

    pub async fn wait_for_events(&self, expected_count: usize) -> Vec<WatcherEvent> {
        let result = timeout(Duration::from_secs(2), async {
            loop {
                let events = self.coordinator.ask(Events).send().await.unwrap();
                if events.len() >= expected_count {
                    return events;
                }
                sleep(Duration::from_millis(20)).await;
            }
        })
        .await;

        result.expect("Timeout waiting for Watcher events")
    }

    pub async fn wait_for_condition<F>(&self, predicate: F) -> Vec<WatcherEvent>
    where
        F: Fn(&WatcherEvent) -> bool,
    {
        let result = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let events = self.coordinator.ask(Events).send().await.unwrap();
                if events.iter().any(&predicate) {
                    return events;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await;

        result.expect("Timeout waiting for specific event condition")
    }
}
