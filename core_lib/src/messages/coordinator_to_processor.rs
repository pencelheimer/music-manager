use async_trait::async_trait;
use kameo::{Actor, actor::ActorRef, error::SendError, prelude::Message};

use crate::models::Track;

/// Sent by the Coordinator to instruct a plugin to start processing a track.
#[derive(Debug, Clone)]
pub struct ProcessTrack {
    /// The full track domain model.
    pub track: Track,
}

/// Sent by the Coordinator to instruct a plugin to resume processing
/// after a user has resolved a pending interaction.
#[derive(Debug, Clone)]
pub struct ResumeProcessing {
    /// The full track domain model.
    pub track: Track,

    /// The data provided by the user in response to the interaction request.
    /// (e.g., `{"selected_index": 1}` or `{"input": "Daft Punk"}`).
    pub user_response: serde_json::Value,
}

pub trait ProcessingHandler:
    Message<ProcessTrack, Reply = ()> + Message<ResumeProcessing, Reply = ()>
{
}
impl<T> ProcessingHandler for T where
    T: Message<ProcessTrack, Reply = ()> + Message<ResumeProcessing, Reply = ()>
{
}

/// Dynamic trait to send processing messages to plugins.
#[async_trait]
pub trait TrackProcessor: Send + Sync {
    async fn send_process_track(&self, track: Track) -> Result<(), SendError<ProcessTrack>>;
    async fn send_resume_processing(
        &self,
        track: Track,
        user_response: serde_json::Value,
    ) -> Result<(), SendError<ResumeProcessing>>;
    async fn graceful_shutdown(&self);
}

/// Blanket implementation for processing plugins.
#[async_trait]
impl<A> TrackProcessor for ActorRef<A>
where
    A: Actor + ProcessingHandler,
{
    async fn send_process_track(&self, track: Track) -> Result<(), SendError<ProcessTrack>> {
        let msg = ProcessTrack { track };

        self.tell(msg).send().await
    }

    async fn send_resume_processing(
        &self,
        track: Track,
        user_response: serde_json::Value,
    ) -> Result<(), SendError<ResumeProcessing>> {
        let msg = ResumeProcessing {
            track,
            user_response,
        };

        self.tell(msg).send().await
    }

    async fn graceful_shutdown(&self) {
        // NOTE(pencelheimer): ignoring if plugin can't stop, as system will shutdown anyway
        let _ = self.stop_gracefully().await;
    }
}
