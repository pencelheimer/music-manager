use core_lib::models::{InteractionType, PendingInteraction};
use sqlx::{Result, SqlitePool, types::Json};

/// Get's interaction for the Track
pub async fn get_by_track_id(
    pool: &SqlitePool,
    track_id: i64,
) -> Result<Option<PendingInteraction>> {
    sqlx::query_as::<_, PendingInteraction>(
        r#"
        SELECT * FROM pending_interactions
        WHERE track_id = $1
        "#,
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await
}

/// Creates new interaction request and updates the track status.
pub async fn pause_processing(
    pool: &SqlitePool,
    track_id: i64,
    plugin_name: &str,
    interaction_type: InteractionType,
    payload: serde_json::Value,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        UPDATE tracks
        SET status = 'paused-waiting-user'
        WHERE id = $1
        "#,
    )
    .bind(track_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO pending_interactions (track_id, plugin_name, interaction_type, payload)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(track_id)
    .bind(plugin_name)
    .bind(interaction_type)
    .bind(Json(payload))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Deletes pending interaction record and updates the status of a track.
pub async fn resolve_interaction(pool: &SqlitePool, track_id: i64) -> Result<()> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query("DELETE FROM pending_interactions WHERE track_id = $1")
        .bind(track_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() > 0 {
        sqlx::query(
            r#"
            UPDATE tracks
            SET status = 'processing'
            WHERE id = $1
            "#,
        )
        .bind(track_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
