use core_lib::messages::{TrackReadyToProcess, TrackRemoved};
use kameo::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum WatcherEvent {
    Ready(PathBuf),
    Removed(PathBuf),
}

#[derive(Actor, Default)]
pub struct MockCoordinator {
    pub received_events: Vec<WatcherEvent>,
}

impl Message<TrackReadyToProcess> for MockCoordinator {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: TrackReadyToProcess,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.received_events.push(WatcherEvent::Ready(msg.0));
    }
}

impl Message<TrackRemoved> for MockCoordinator {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: TrackRemoved,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.received_events.push(WatcherEvent::Removed(msg.0));
    }
}

pub struct Events;

impl Message<Events> for MockCoordinator {
    type Reply = Vec<WatcherEvent>;

    async fn handle(&mut self, _msg: Events, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.received_events.clone()
    }
}
