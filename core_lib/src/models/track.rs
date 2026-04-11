use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use sqlx::{FromRow, Type};
use time::OffsetDateTime;
use tokio::fs;

/// Represents the current status of a track in the pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Type)]
#[sqlx(rename_all = "kebab-case")]
pub enum TrackStatus {
    Pending,
    Processing,
    PausedWaitingUser,
    Completed,
    Failed,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct FileMetrics {
    /// The size of the file in bytes.
    pub file_size: i64,

    /// The time the file was last modified.
    pub mtime: OffsetDateTime,
}

/// The core model representing a Track.
///
/// This structure is shared across plugins and the coordinator.
#[derive(Debug, Clone, FromRow)]
pub struct Track {
    /// The unique identifier from the database.
    pub id: i64,

    /// The absolute path to the audio file on disk.
    #[sqlx(try_from = "String")]
    pub file_path: PathBuf,

    #[sqlx(flatten)]
    pub metrics: FileMetrics,

    /// The name of the plugin/stage currently processing the track,
    /// or 'init' if it hasn't started yet.
    pub current_stage: String,

    /// The general status of the processing pipeline for this track.
    pub status: TrackStatus,

    /// The time this record was created.
    pub created_at: OffsetDateTime,

    /// The time this record was last updated.
    pub updated_at: OffsetDateTime,
}

impl FileMetrics {
    pub async fn get_for_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let meta = fs::metadata(path.as_ref()).await?;

        let file_size = meta.len() as i64;
        let mtime = OffsetDateTime::from(meta.modified().unwrap_or_else(|_| SystemTime::now()));

        Ok(Self { file_size, mtime })
    }
}
