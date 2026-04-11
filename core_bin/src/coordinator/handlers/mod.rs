pub mod plugins;
pub mod services;

mod watcher {
    mod ready_to_process;
    mod removed;
}

use core_lib::{messages::RegisterPlugin, models::TrackStatus};
use kameo::prelude::*;
use tracing::{error, info, instrument};

use crate::{Coordinator, db::tracks};

impl Coordinator {
    /// Advances the pipeline to the next stage for a given track.
    /// This is called directly during track processing or recovery,
    /// avoiding internal message passing and mailbox races.
    #[instrument(skip_all, fields(track_id = id))]
    pub async fn advance_pipeline(&self, id: i64) {
        let Some(track) = self.get_track(id).await else {
            return;
        };

        if track.status != TrackStatus::Pending && track.status != TrackStatus::Processing {
            return;
        }

        if !tokio::fs::try_exists(&track.file_path)
            .await
            .unwrap_or(false)
        {
            tracing::warn!(
                path = ?track.file_path,
                "File is missing from disk after stage completion. Aborting pipeline."
            );

            let _ = tracks::set_status(&self.db_pool, id, TrackStatus::Missing).await;
            return;
        }

        let Some(next_stage) = self.get_next_stage(&track.current_stage) else {
            info!("Track reached the end of the pipeline. Marking as Completed.");
            if let Err(e) = tracks::set_status(&self.db_pool, id, TrackStatus::Completed).await {
                error!(error = %e, "Failed to mark track as completed");
            }
            return;
        };

        let track = match tracks::update_stage(&self.db_pool, id, &next_stage).await {
            Ok(track) => track,
            Err(e) => {
                error!(error = %e, ?next_stage, "Failed to update track stage in DB");
                return;
            }
        };

        self.with_plugin(next_stage, id, async move |plugin| {
            plugin.send_process_track(track).await
        })
        .await;
    }
}

impl Message<RegisterPlugin> for Coordinator {
    type Reply = ();

    #[instrument(skip_all, fields(name = msg.name))]
    async fn handle(&mut self, msg: RegisterPlugin, _ctx: &mut Context<Self, Self::Reply>) {
        tracing::info!("Registering plugin");
        self.registered_plugins.insert(msg.name, msg.processor);
    }
}
