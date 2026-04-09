use std::path::PathBuf;

use core_lib::messages::{
    TrackCoordinator, TrackReadyToProcess as ReadyToProcess, TrackRemoved as Removed,
};
use kameo::prelude::{ActorRef, Context, Message};
use notify::{
    EventKind, RecommendedWatcher,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer,
};
use tracing::{debug, info, instrument, warn};

use crate::{WatcherService, config::Config, error::WatcherError};

impl<C: TrackCoordinator> WatcherService<C> {
    /// Initializes the debouncer with a closure that forwards events to the actor
    pub fn init_debouncer(
        config: &Config,
        actor_ref: ActorRef<Self>,
    ) -> Result<Debouncer<RecommendedWatcher, RecommendedCache>, WatcherError> {
        let callback = move |result: DebounceEventResult| {
            if let Ok(events) = result {
                let _ = actor_ref.tell(FsDebouncedEvents(events)).blocking_send();
            }
        };

        let mut debouncer = new_debouncer(config.debounce_duration, None, callback)?;
        debouncer.watch(&config.watch_dir, notify::RecursiveMode::Recursive)?;

        Ok(debouncer)
    }

    /// Filters debounced events and routes relevant file changes to the processing logic.
    #[instrument(skip(self, events), fields(events = events.len()))]
    async fn process_events(&self, events: Vec<DebouncedEvent>) {
        for event in events {
            let is_relevent = matches!(
                &event.kind,
                EventKind::Create(CreateKind::File)
                    | EventKind::Remove(RemoveKind::File)
                    | EventKind::Modify(ModifyKind::Name(RenameMode::To))
                    | EventKind::Modify(ModifyKind::Name(RenameMode::Both))
                    | EventKind::Modify(ModifyKind::Data(_))
            );

            if is_relevent {
                self.send_events_for_paths(event.event.paths).await;
            };
        }
    }

    /// Checks extensions and existence for a list of paths before notifying the target actor.
    async fn send_events_for_paths(&self, paths: Vec<PathBuf>) {
        for path in paths {
            debug!("Debouncer stabilized file: {:?}", path);

            let extension_allowed = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| self.allowed_extensions.contains(ext));

            if !extension_allowed {
                continue;
            }

            if tokio::fs::try_exists(&path).await.unwrap_or(false) {
                self.send_ready(path).await;
            } else {
                self.send_removed(path).await;
            }
        }
    }

    /// Sends a ReadyToProcess message to the target coordinator.
    #[instrument(skip_all, fields(path = ?path))]
    async fn send_ready(&self, path: PathBuf) {
        info!("Notifying coordinator: file ready");
        let res = self.target_actor.tell(ReadyToProcess(path)).send().await;

        if let Err(e) = res {
            warn!("Failed to send ReadyToProcess to target actor: {}", e);
        }
    }

    /// Sends a TrackRemoved message to the target coordinator.
    #[instrument(skip_all, fields(path = ?path))]
    async fn send_removed(&self, path: PathBuf) {
        info!("Notifying coordinator: file removed");
        let res = self.target_actor.tell(Removed(path)).send().await;

        if let Err(e) = res {
            warn!("Failed to send TrackRemoved to target actor: {}", e);
        }
    }
}

/// A message sent internally to process debounced events within the actor's context.
pub struct FsDebouncedEvents(pub Vec<DebouncedEvent>);

impl<C: TrackCoordinator> Message<FsDebouncedEvents> for WatcherService<C> {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: FsDebouncedEvents,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.process_events(msg.0).await;
    }
}
