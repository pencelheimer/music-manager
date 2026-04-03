use core_lib::messages::{TrackReadyToProcess as ReadyToProcess, TrackRemoved as Removed};
use kameo::{
    Actor,
    message::{Context, Message},
};
use tracing::info;

/// Main system orchestrator
///
/// Accepts messages from background services and controls tracs lifecycle.
#[derive(Debug, Actor)]
pub struct CoordinatorActor;

impl Message<ReadyToProcess> for CoordinatorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        ReadyToProcess(path): ReadyToProcess,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("📥 Received signal to process track: {:?}", path);
        // TODO(pencelheimer): DB check, hashing, tasks creation
    }
}

impl Message<Removed> for CoordinatorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        Removed(path): Removed,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("🗑️ Received signal that track was removed: {:?}", path);
        // TODO(pencelheimer): DB status update, aborting of dependent tasks
    }
}
