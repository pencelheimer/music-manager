use std::path::PathBuf;

use core_lib::messages::{
    TrackEventHandler, TrackReadyToProcess as ReadyToProcess, TrackRemoved as Removed,
};
use kameo::Actor;
use notify::{
    EventKind,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use notify_debouncer_full::{DebounceEventHandler, DebounceEventResult, DebouncedEvent};
use tracing::{debug, instrument, warn};

use crate::WatcherService;

impl<A: Actor + TrackEventHandler> WatcherService<A> {
    #[instrument(skip(self, events), fields(events = events.len()))]
    fn process_events(&self, events: Vec<DebouncedEvent>) {
        for event in events {
            let event = match &event.kind {
                EventKind::Create(CreateKind::File)
                | EventKind::Remove(RemoveKind::File)
                | EventKind::Modify(ModifyKind::Name(RenameMode::To))
                | EventKind::Modify(ModifyKind::Name(RenameMode::Both))
                | EventKind::Modify(ModifyKind::Data(_)) => event.event,
                kind => {
                    debug!("Skipping event {kind:?}");
                    continue;
                }
            };

            self.send_events_for_paths(event.paths);
        }
    }

    fn send_events_for_paths(&self, paths: Vec<PathBuf>) {
        for path in paths {
            debug!("Debouncer stabilized file: {:?}", path);

            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_none_or(|ext| !self.allowed_extensions.iter().any(|e| e == ext))
            {
                continue;
            }

            if path.exists() {
                self.send_ready(path);
            } else {
                self.send_removed(path);
            }
        }
    }

    fn send_ready(&self, path: PathBuf) {
        let req = self.target_actor.tell(ReadyToProcess(path)).blocking_send();

        if let Err(e) = req {
            warn!("Failed to send ReadyToProcess: {}", e);
        }
    }

    fn send_removed(&self, path: PathBuf) {
        let req = self.target_actor.tell(Removed(path)).blocking_send();

        if let Err(e) = req {
            warn!("Failed to send Removed: {}", e);
        }
    }
}

impl<A: Actor + TrackEventHandler> DebounceEventHandler for WatcherService<A> {
    fn handle_event(&mut self, event: DebounceEventResult) {
        match event {
            Ok(events) => self.process_events(events),
            Err(e) => warn!("FS Watcher error: {:?}", e),
        }
    }
}
