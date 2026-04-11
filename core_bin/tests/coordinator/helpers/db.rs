use std::path::Path;

use core_bin::db::{interactions, tracks};
use core_lib::models::{FileMetrics, PendingInteraction, Track, TrackStatus};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::TestApp;
pub async fn seed_track_no_metrics(
    db_pool: &SqlitePool,
    path: impl AsRef<Path>,
    status: TrackStatus,
    stage: impl ToString,
) -> Track {
    let metrics = FileMetrics {
        file_size: 0,
        mtime: OffsetDateTime::now_utc(),
    };

    seed_track(db_pool, path, &metrics, status, stage).await
}
pub async fn seed_track(
    db_pool: &SqlitePool,
    path: impl AsRef<Path>,
    metrics: &FileMetrics,
    status: TrackStatus,
    stage: impl ToString,
) -> Track {
    sqlx::query_as::<_, Track>(
        r#"
        INSERT INTO tracks (file_path, mtime, file_size, status, current_stage)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(path.as_ref().to_string_lossy())
    .bind(metrics.mtime)
    .bind(metrics.file_size)
    .bind(status)
    .bind(stage.to_string())
    .fetch_one(db_pool)
    .await
    .expect("DB query failed")
}

impl TestApp {
    pub async fn get_track_by_path(&self, path: impl AsRef<Path>) -> Option<Track> {
        tracks::find_by_path(&self.db_pool, path.as_ref())
            .await
            .expect("DB query failed")
    }

    pub async fn get_track_by_id(&self, id: i64) -> Option<Track> {
        tracks::find_by_id(&self.db_pool, id)
            .await
            .expect("DB query failed")
    }

    pub async fn get_track_pending_interaction(&self, id: i64) -> Option<PendingInteraction> {
        interactions::get_by_track_id(&self.db_pool, id)
            .await
            .expect("DB query failed")
    }
}
