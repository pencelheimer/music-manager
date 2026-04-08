use core_lib::{
    db,
    messages::{
        InteractionResolved, StageCompleted, StageFailed, TrackReadyToProcess, TrackRemoved,
    },
    models::TrackStatus,
};
use serde_json::json;
use sqlx::SqlitePool;
use tempfile::NamedTempFile;

use crate::{Processed, Resumed, TestApp};

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_edge_stage_failed(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(path.to_path_buf()).await.unwrap();

    let processed_before = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_before, 1);

    let error_msg = "Fatal error".to_string();

    app.coordinator_send_sync(StageFailed {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        error_message: error_msg.clone(),
    })
    .await;

    let processed_after = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_before, processed_after);

    let failed_track = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(failed_track.status, TrackStatus::Failed);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_edge_file_deleted_during_processing_aborts_pipeline(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    app.coordinator_send_sync(TrackReadyToProcess(path.clone()))
        .await;
    let track = app.get_track_by_path(path.clone()).await.unwrap();

    drop(temp_file);
    app.coordinator_send_sync(TrackRemoved(path.clone())).await;

    let track_after_remove = app.get_track_by_path(path.clone()).await.unwrap();
    assert_eq!(track_after_remove.status, TrackStatus::Processing);

    app.coordinator_send_sync(StageCompleted {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        new_metrics: None,
        new_path: None,
    })
    .await;

    let processed_total = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_total, 1);

    let final_track = app.get_track_by_path(path).await.unwrap();
    assert_eq!(final_track.status, TrackStatus::Missing);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_edge_duplicate_ready_event_ignored(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;

    let processed_first = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_first, 1);

    let track_initial = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(track_initial.status, TrackStatus::Processing);

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;

    let processed_second = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_first, processed_second);

    let track_after_duplicate = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(track_after_duplicate.id, track_initial.id);
    assert_eq!(track_after_duplicate.status, TrackStatus::Processing);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_edge_late_stage_completed_ignored(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(path.to_path_buf()).await.unwrap();

    let processed_initial = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_initial, 1);

    app.coordinator_send_sync(StageFailed {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        error_message: "Manual abort".to_string(),
    })
    .await;

    let track_failed = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(track_failed.status, TrackStatus::Failed);

    app.coordinator_send_sync(StageCompleted {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        new_metrics: None,
        new_path: None,
    })
    .await;

    let processed_after_late_msg = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_initial, processed_after_late_msg);

    let track_final = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(track_final.status, TrackStatus::Failed);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_edge_phantom_interaction_resolved_ignored(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(path.to_path_buf()).await.unwrap();

    let processed_before = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_before, 1, "Track should be processing in stage_1");
    assert_eq!(track.status, TrackStatus::Processing);

    let phantom_response = json!({"input": "Some phantom data"});

    app.coordinator_send_sync(InteractionResolved {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        response_payload: phantom_response,
    })
    .await;

    let track_after = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(track_after.status, TrackStatus::Processing);

    let resumed_state = app.pipeline.ask(Resumed).send().await.unwrap();
    assert_eq!(resumed_state.len(), 0);
}
