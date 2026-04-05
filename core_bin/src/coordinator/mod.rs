mod handlers;

use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::Result;
use core_lib::{
    GlobalState,
    messages::TrackProcessor,
    models::{FileMetrics, Track, TrackStatus},
};
use kameo::{Actor, actor::ActorRef};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::fs;
use tracing::{error, info};

use crate::db::tracks;

/// Main system orchestrator
///
/// Accepts messages from background services and controls tracks lifecycle.
pub struct Coordinator {
    pub db_pool: SqlitePool,
    pub pipeline_steps: Vec<String>,
    pub registered_plugins: HashMap<String, Arc<dyn TrackProcessor>>,
}

impl Actor for Coordinator {
    type Args = Self;
    type Error = anyhow::Error;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> anyhow::Result<Self> {
        let orphaned_tracks = tracks::get_by_statuses(
            &args.db_pool,
            &[TrackStatus::Processing, TrackStatus::Pending],
        )
        .await?;

        for track in orphaned_tracks {
            info!(track_id = track.id, stage = ?track.current_stage, "Recovering track processing");

            args.with_plugin(track.current_stage.clone(), track.id, async move |plugin| {
                plugin.send_process_track(track).await
            })
            .await;
        }

        Ok(args)
    }
}

impl Coordinator {
    pub fn new(
        state: GlobalState,
        registered_plugins: HashMap<String, Arc<dyn TrackProcessor>>,
    ) -> Result<Self> {
        let GlobalState {
            lua_vm, db_pool, ..
        } = state;

        // NOTE(pencelheimer): just register tracks if pipeline is not specified
        lua_vm.set_default_config_value("pipeline_steps", Vec::<String>::new())?;

        let pipeline_steps = lua_vm.pipeline_steps()?;

        Ok(Self {
            db_pool,
            pipeline_steps,
            registered_plugins,
        })
    }

    pub fn get_next_stage(&self, current_stage: impl AsRef<str>) -> Option<String> {
        let current_stage = current_stage.as_ref();

        if current_stage == "init" {
            return self.pipeline_steps.first().cloned();
        }

        self.pipeline_steps
            .iter()
            .position(|stage| stage == current_stage)
            .and_then(|i| self.pipeline_steps.get(i + 1).cloned())
    }

    /// Gets a track from the DB.
    /// Logs errors and returns None if track was not found or DB error occurred.
    async fn get_track(&self, track_id: i64) -> Option<Track> {
        match tracks::find_by_id(&self.db_pool, track_id).await {
            Ok(Some(track)) => Some(track),
            Ok(None) => {
                error!(track_id, "Track not found in database.");
                None
            }
            Err(e) => {
                error!(error = %e, track_id, "Failed to fetch track from database.");
                None
            }
        }
    }

    // NOTE(pencelheimer): F is an async closure that accepts the plugin ref and returns the Result<(), Error>
    async fn with_plugin<F, Fut, E>(&self, plugin_name: impl AsRef<str>, track_id: i64, action: F)
    where
        F: FnOnce(Arc<dyn TrackProcessor>) -> Fut,
        Fut: Future<Output = Result<(), E>> + Send,
        E: std::error::Error + 'static,
    {
        let Some(plugin) = self.registered_plugins.get(plugin_name.as_ref()) else {
            error!("Plugin not found in coordinator registry.");
            let _ = tracks::set_status(&self.db_pool, track_id, TrackStatus::Failed).await;
            return;
        };

        if let Err(e) = action(plugin.clone()).await {
            error!(error = %e, "Failed to send message to plugin");
            let _ = tracks::set_status(&self.db_pool, track_id, TrackStatus::Failed).await;
            return;
        }

        info!("Successfully dispatched message to plugin.");
    }
}

/// Get the File Metrics from the file system
pub async fn extract_metrics(path: impl AsRef<Path>) -> Result<FileMetrics> {
    let metadata = fs::metadata(path.as_ref()).await?;

    let file_size = metadata.len() as i64;
    let mtime = metadata.modified().map(OffsetDateTime::from)?;

    Ok(FileMetrics { file_size, mtime })
}
