use core_lib::{
    messages::{ProcessTrack, ResumeProcessing},
    models::Track,
};
use kameo::prelude::*;

/// Special Mock Actor that can act as any plugin in the pipeline and used to record actions
#[derive(Actor, Default)]
pub struct MockPipeline {
    pub processed_tracks: Vec<Track>,
    pub resumed_tracks: Vec<(Track, serde_json::Value)>,
}

impl MockPipeline {
    pub fn spawn_default() -> ActorRef<Self> {
        Self::spawn(Self::default())
    }
}

/// Special gate message for test sync
pub struct Processed;

impl Message<Processed> for MockPipeline {
    type Reply = Vec<Track>;

    async fn handle(
        &mut self,
        _msg: Processed,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.processed_tracks.clone()
    }
}

impl Message<ProcessTrack> for MockPipeline {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ProcessTrack,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.processed_tracks.push(msg.track);
    }
}

impl Message<ResumeProcessing> for MockPipeline {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ResumeProcessing,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.resumed_tracks.push((msg.track, msg.user_response));
    }
}

/// Special gate message for test sync
pub struct Resumed;

impl Message<Resumed> for MockPipeline {
    type Reply = Vec<(Track, serde_json::Value)>;

    async fn handle(
        &mut self,
        _msg: Resumed,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.resumed_tracks.clone()
    }
}
