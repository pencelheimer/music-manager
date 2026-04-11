use std::path::Path;

use core_lib::models::{FileMetrics, Track, TrackStatus};
use sqlx::{QueryBuilder, Result, SqlitePool};

/// Finds Track by ID.
pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Track>> {
    sqlx::query_as::<_, Track>("SELECT * FROM tracks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Finds Track by path.
pub async fn find_by_path(pool: &SqlitePool, path: impl AsRef<Path>) -> Result<Option<Track>> {
    sqlx::query_as::<_, Track>(
        r#"
        SELECT *
        FROM tracks
        WHERE file_path = $1
        LIMIT 1
        "#,
    )
    .bind(path.as_ref().to_str())
    .fetch_optional(pool)
    .await
}

/// Finds Track by pseudo hash (size + mtime).
pub async fn find_by_pseudo_hash(
    pool: &SqlitePool,
    metrics: &FileMetrics,
) -> Result<Option<Track>> {
    sqlx::query_as::<_, Track>(
        r#"
        SELECT *
        FROM tracks
        WHERE file_size = $1 AND mtime = $2
        LIMIT 1
        "#,
    )
    .bind(metrics.file_size)
    .bind(metrics.mtime)
    .fetch_optional(pool)
    .await
}

/// Gets all Tracks by a list of statuses
pub async fn get_by_statuses(
    pool: &SqlitePool,
    statuses: impl AsRef<[TrackStatus]>,
) -> Result<Vec<Track>> {
    let statuses = statuses.as_ref();

    if statuses.is_empty() {
        return Ok(vec![]);
    }

    let mut query = QueryBuilder::new("SELECT * FROM tracks WHERE status IN (");

    let mut separated = query.separated(", ");
    for status in statuses {
        separated.push_bind(status.clone());
    }
    separated.push_unseparated(")");

    query.build_query_as::<Track>().fetch_all(pool).await
}

/// Updates the path of the Track
pub async fn update_path(pool: &SqlitePool, id: i64, new_path: impl AsRef<Path>) -> Result<Track> {
    let path = new_path.as_ref().to_string_lossy();

    sqlx::query_as::<_, Track>(
        r#"
        UPDATE tracks
        SET file_path = $1
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(path)
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Updates the current stage of the Track
pub async fn update_stage(pool: &SqlitePool, id: i64, new_stage: impl AsRef<str>) -> Result<Track> {
    let stage = new_stage.as_ref();

    sqlx::query_as::<_, Track>(
        r#"
        UPDATE tracks
        SET
            current_stage = $1,
            status = "processing"
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(stage)
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Creates new Track or updates existing one, resetting status.
pub async fn insert_new(
    pool: &SqlitePool,
    path: impl AsRef<Path>,
    metrics: &FileMetrics,
) -> Result<Track> {
    let path_str = path.as_ref().to_string_lossy();

    sqlx::query_as::<_, Track>(
        r#"
        INSERT INTO tracks (file_path, file_size, mtime, current_stage, status)
        VALUES ($1, $2, $3, 'init', 'pending')
        RETURNING *
        "#,
    )
    .bind(path_str)
    .bind(metrics.file_size)
    .bind(metrics.mtime)
    .fetch_one(pool)
    .await
}

/// Reset the status of a track.
pub async fn reset(pool: &SqlitePool, id: i64, metrics: &FileMetrics) -> Result<Track> {
    sqlx::query_as::<_, Track>(
        r#"
        UPDATE tracks SET
            file_size = $1,
            mtime = $2,
            current_stage = 'init',
            status = 'pending'
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(metrics.file_size)
    .bind(metrics.mtime)
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Set the new status for a track.
pub async fn set_status(pool: &SqlitePool, id: i64, status: TrackStatus) -> Result<Track> {
    sqlx::query_as::<_, Track>(
        r#"
        UPDATE tracks
        SET status = $1
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Applies changes made by the plugin
pub async fn apply_modifications(
    pool: &SqlitePool,
    id: i64,
    new_path: Option<&Path>,
    new_metrics: Option<&FileMetrics>,
) -> Result<()> {
    let path_str = new_path.map(|p| p.to_string_lossy());

    let (new_size, new_mtime) = match new_metrics {
        Some(m) => (Some(m.file_size), Some(m.mtime)),
        None => (None, None),
    };

    sqlx::query(
        r#"
        UPDATE tracks
        SET
            file_path = COALESCE($1, file_path),
            file_size = COALESCE($2, file_size),
            mtime = COALESCE($3, mtime)
        WHERE id = $4
        "#,
    )
    .bind(path_str)
    .bind(new_size)
    .bind(new_mtime)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}
