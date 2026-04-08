use std::path::Path;

use anyhow::Result;
use core_lib::{messages::TrackRemoved as Removed, models::TrackStatus};
use kameo::prelude::{Context, Message};
use tracing::{debug, error, info, instrument, warn};

use crate::{coordinator::Coordinator, db::tracks};

impl Coordinator {
    #[instrument(skip_all, fields(path = ?path.as_ref()))]
    async fn handle_removal(&self, path: impl AsRef<Path>) -> Result<()> {
        let Some(track) = tracks::find_by_path(&self.db_pool, path).await? else {
            debug!("Removed untracked file. Ignoring");
            return Ok(());
        };

        if track.status == TrackStatus::Processing {
            warn!("File removed while being processed. Plugin may recover it. Ignoring");
            return Ok(());
        }

        info!(track_id = track.id, "Marking track as missing");
        tracks::set_status(&self.db_pool, track.id, TrackStatus::Missing).await?;

        Ok(())
    }
}

impl Message<Removed> for Coordinator {
    type Reply = ();

    #[instrument(skip_all, fields(path = ?path))]
    async fn handle(
        &mut self,
        Removed(path): Removed,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Err(e) = self.handle_removal(&path).await {
            error!(error = %e, "Failed to handle track removal");
        }
    }
}
