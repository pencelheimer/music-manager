use core_lib::{db, models::TrackStatus};
use sqlx::SqlitePool;
use tempfile::NamedTempFile;

use crate::{Processed, Ping, TestApp, db::seed_track_no_metrics};

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_recovery_on_start(pool: SqlitePool) {
    let stages = ["stage_1", "stage_2"];

    let pending_path = NamedTempFile::new().unwrap().into_temp_path();
    let processing_path = NamedTempFile::new().unwrap().into_temp_path();
    let waiting_path = NamedTempFile::new().unwrap().into_temp_path();

    seed_track_no_metrics(&pool, &pending_path, TrackStatus::Pending, "init").await;
    seed_track_no_metrics(&pool, &processing_path, TrackStatus::Processing, "stage_1").await;
    seed_track_no_metrics(
        &pool,
        &waiting_path,
        TrackStatus::PausedWaitingUser,
        "stage_1",
    )
    .await;

    let app = TestApp::new(pool.clone(), stages);
    app.coordinator.ask(Ping).send().await.unwrap();

    let processed = app.pipeline.ask(Processed).send().await.unwrap();
    assert_eq!(processed.len(), 2);

    let track_a = processed
        .iter()
        .find(|t| t.id == 1)
        .expect("Track A should be recovered");
    assert_eq!(track_a.current_stage, "stage_1");

    let track_b = processed
        .iter()
        .find(|t| t.id == 2)
        .expect("Track B should be recovered");
    assert_eq!(track_b.current_stage, "stage_1");

    let track_v_in_db = app.get_track_by_id(3).await.unwrap();
    assert_eq!(track_v_in_db.status, TrackStatus::PausedWaitingUser);
}
