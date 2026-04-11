use core_lib::{
    messages::{CoordinatorForProcessor, ResumeProcessing},
    models::{FileMetrics, Track},
};
use kameo::prelude::{Context, Message};
use tracing::{error, info, instrument};

use crate::{acoustid::TrackMatch, actor::MetadataRestorer, error::PluginError};

impl<C: CoordinatorForProcessor> MetadataRestorer<C> {
    async fn handle_resume_processing(
        &self,
        track: &Track,
        user_response: serde_json::Value,
    ) -> Result<(), PluginError> {
        let selected_candidate: TrackMatch = serde_json::from_value(user_response)?;

        self.handle_candidate(track, &selected_candidate).await?;

        let new_metrics = Some(FileMetrics::get_for_file(&track.file_path).await?);
        self.send_stage_completed(track.id, new_metrics).await?;

        Ok(())
    }
}

impl<C: CoordinatorForProcessor> Message<ResumeProcessing> for MetadataRestorer<C> {
    type Reply = ();

    #[instrument(skip_all, fields(track_id = msg.track.id, path = %msg.track.file_path.display()))]
    async fn handle(
        &mut self,
        msg: ResumeProcessing,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let ResumeProcessing {
            track,
            user_response,
        } = msg;
        info!("Resuming track processing after user interaction");

        if let Err(e) = self.handle_resume_processing(&track, user_response).await {
            error!("Plugin failed during processing: {}", e);
            let _ = self.send_stage_failed(track.id, e).await;
        }
    }
}
