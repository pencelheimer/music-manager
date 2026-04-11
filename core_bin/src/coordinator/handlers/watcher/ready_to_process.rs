use std::path::Path;

use anyhow::Result;
use core_lib::{
    messages::TrackReadyToProcess as ReadyToProcess,
    models::{FileMetrics, Track, TrackStatus},
};
use kameo::prelude::{Context, Message};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    coordinator::{Coordinator, extract_metrics},
    db::tracks,
};

impl Coordinator {
    #[instrument(
        skip_all,
        fields(new_path = ?path.as_ref(), file_size = ?metrics.file_size, mtime = ?metrics.mtime)
    )]
    async fn sync_track(
        &self,
        path: impl AsRef<Path>,
        metrics: FileMetrics,
    ) -> Result<Option<Track>> {
        let path = path.as_ref();

        if let Some(track) = tracks::find_by_path(&self.db_pool, path).await? {
            return self.sync_track_by_path(track, metrics).await;
        }

        if let Some(track) = tracks::find_by_pseudo_hash(&self.db_pool, &metrics).await? {
            return self.sync_track_by_pseudo_hash(track, path).await;
        }

        info!("Discovered new track, registering in database.");
        let inserted = tracks::insert_new(&self.db_pool, path, &metrics).await?;
        Ok(Some(inserted))
    }

    #[instrument(skip_all, fields(old_path = ?track.file_path))]
    async fn sync_track_by_path(
        &self,
        track: Track,
        metrics: FileMetrics,
    ) -> Result<Option<Track>> {
        if track.metrics == metrics {
            debug!("Track metrics match. No action needed.");
            return Ok(None);
        }

        if track.status == TrackStatus::Processing {
            debug!("Track is currently processed by a plugin. Ignoring FS event.");
            return Ok(None);
        }

        info!("File modified externally. Resetting track.");
        let updated = tracks::reset(&self.db_pool, track.id, &metrics).await?;
        return Ok(Some(updated));
    }

    #[instrument(skip_all)]
    async fn sync_track_by_pseudo_hash(
        &self,
        track: Track,
        path: impl AsRef<Path>,
    ) -> Result<Option<Track>> {
        let updated = tracks::update_path(&self.db_pool, track.id, path.as_ref()).await?;

        match updated.status {
            TrackStatus::Pending | TrackStatus::PausedWaitingUser => return Ok(Some(updated)),
            _ => {
                debug!("Moved track is already completed or failed.");
                return Ok(None);
            }
        }
    }
}

impl Message<ReadyToProcess> for Coordinator {
    type Reply = ();

    #[instrument(skip_all, fields(path = ?path))]
    async fn handle(
        &mut self,
        ReadyToProcess(path): ReadyToProcess,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("Processing new track signal");

        let metrics = match extract_metrics(&path).await {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, "Failed to read metadata, file might have been deleted mid-flight");
                return;
            }
        };

        let track_to_process = match self.sync_track(&path, metrics).await {
            Ok(Some(track)) => track,
            Ok(None) => return,
            Err(e) => {
                error!(error = %e, "Failed to sync track state with database");
                return;
            }
        };

        self.advance_pipeline(track_to_process.id).await;
    }
}
