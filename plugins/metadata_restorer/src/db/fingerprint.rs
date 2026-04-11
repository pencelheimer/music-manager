use sqlx::SqlitePool;

use crate::{chromaprint::Fingerprint, error::PluginError};

pub async fn insert(pool: &SqlitePool, track_id: i64, fp: &Fingerprint) -> Result<(), PluginError> {
    sqlx::query(
        r#"
        INSERT INTO track_fingerprints (track_id, chromaprint, duration)
        VALUES (?, ?, ?)
        ON CONFLICT(track_id) DO UPDATE SET
            chromaprint = excluded.chromaprint,
            duration = excluded.duration,
            mbid = NULL
        "#,
    )
    .bind(track_id)
    .bind(&fp.fingerprint)
    .bind(fp.duration)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_mbid(
    pool: &SqlitePool,
    track_id: i64,
    mbid: impl AsRef<str>,
) -> Result<(), PluginError> {
    sqlx::query("UPDATE track_fingerprints SET mbid = ? WHERE track_id = ?")
        .bind(mbid.as_ref())
        .bind(track_id)
        .execute(pool)
        .await?;

    Ok(())
}
