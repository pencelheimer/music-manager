use core_lib::{
    messages::{CoordinatorForProcessor, ProcessTrack},
    models::{FileMetrics, Track},
};
use kameo::prelude::{Context, Message};
use tracing::{error, info, instrument, warn};

use crate::{
    actor::MetadataRestorer, chromaprint::calculate_fingerprint, db::fingerprint,
    error::PluginError,
};

impl<C: CoordinatorForProcessor> MetadataRestorer<C> {
    #[instrument(skip_all)]
    async fn handle_process_track(&self, track: &Track) -> Result<(), PluginError> {
        let fp = calculate_fingerprint(&track.file_path).await?;
        fingerprint::insert(&self.db_pool, track.id, &fp).await?;

        let candidates = self.aid_client.lookup(&fp).await?;
        match candidates.len() {
            0 => {
                warn!("No matches found in AcoustID for this track");
                self.send_stage_completed(track.id, None).await?;
            }
            1 => {
                info!("Found exact match");

                self.handle_candidate(track, &candidates[0]).await?;
                let new_metrics = Some(FileMetrics::get_for_file(&track.file_path).await?);
                self.send_stage_completed(track.id, new_metrics).await?;
            }
            _ => {
                info!("Found multiple candidates. User interaction required.",);
                let payload = serde_json::to_value(&candidates)?;
                self.send_stage_paused(track.id, payload).await?;
            }
        }

        Ok(())
    }
}

impl<C: CoordinatorForProcessor> Message<ProcessTrack> for MetadataRestorer<C> {
    type Reply = ();

    #[instrument(skip_all, fields(track_id = msg.track.id, path = %msg.track.file_path.display()))]
    async fn handle(
        &mut self,
        msg: ProcessTrack,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let ProcessTrack { track } = msg;
        info!("Started processing track");

        if let Err(e) = self.handle_process_track(&track).await {
            error!("Plugin failed during processing: {}", e);
            let _ = self.send_stage_failed(track.id, e).await;
        }
    }
}
