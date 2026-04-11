use core_lib::{
    db,
    messages::{InteractionResolved, StageCompleted, TrackReadyToProcess, UserInteractionRequired},
    models::{InteractionType, TrackStatus},
};
use serde_json::json;
use sqlx::SqlitePool;
use tempfile::NamedTempFile;

use crate::{Processed, Resumed, TestApp};

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_happy_path_start_pipeline(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let processed = app.pipeline.ask(Processed).send().await.unwrap();

    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].current_stage, "stage_1");

    let track = app.get_track_by_path(path).await.unwrap();

    assert_eq!(track.status, TrackStatus::Processing);
    assert_eq!(track.current_stage, "stage_1");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_happy_path_advance_pipeline(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2", "stage_3"]);
    let initial_path = NamedTempFile::new().unwrap().into_temp_path();
    let new_path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(initial_path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(&initial_path).await.unwrap();
    assert_eq!(track.current_stage, "stage_1");

    app.coordinator_send_sync(StageCompleted {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        new_metrics: None,
        new_path: Some(new_path.to_path_buf()),
    })
    .await;

    let processed = app.pipeline.ask(Processed).send().await.unwrap();
    assert_eq!(processed.len(), 2);
    assert_eq!(processed[1].current_stage, "stage_2");
    assert_eq!(processed[1].file_path, new_path.to_path_buf());

    let updated_track = app.get_track_by_id(track.id).await.unwrap();
    assert_eq!(updated_track.status, TrackStatus::Processing);
    assert_eq!(updated_track.current_stage, "stage_2");
    assert_eq!(updated_track.file_path, new_path.to_path_buf());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_happy_path_complete_pipeline(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(path.to_path_buf()).await.unwrap();

    app.coordinator_send_sync(StageCompleted {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        new_metrics: None,
        new_path: None,
    })
    .await;

    app.coordinator_send_sync(StageCompleted {
        track_id: track.id,
        plugin_name: "stage_2".to_string(),
        new_metrics: None,
        new_path: None,
    })
    .await;

    let processed_count = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_count, 2);

    let processed_track = app.get_track_by_path(path).await.unwrap();
    assert_eq!(processed_track.status, TrackStatus::Completed);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_interaction_required_pauses_pipeline(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    let processed_before = app.pipeline.ask(Processed).send().await.unwrap().len();

    let interaction_payload = json!({
        "prompt": "Which Daft Punk song is this?",
        "options": ["Get Lucky", "Harder, Better, Faster, Stronger"]
    });

    app.coordinator_send_sync(UserInteractionRequired {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        interaction_type: InteractionType::SelectFromList,
        request_payload: interaction_payload.clone(),
    })
    .await;

    let processed_after = app.pipeline.ask(Processed).send().await.unwrap().len();
    assert_eq!(processed_before, processed_after);

    let processed_track = app.get_track_by_path(path).await.unwrap();
    assert_eq!(processed_track.status, TrackStatus::PausedWaitingUser);

    let pending_interaction = app.get_track_pending_interaction(track.id).await.unwrap();
    assert_eq!(pending_interaction.payload.0, interaction_payload);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn test_coordinator_interaction_resolved_resumes_pipeline(pool: SqlitePool) {
    let app = TestApp::new(pool.clone(), ["stage_1", "stage_2"]);
    let path = NamedTempFile::new().unwrap().into_temp_path();

    app.coordinator_send_sync(TrackReadyToProcess(path.to_path_buf()))
        .await;
    let track = app.get_track_by_path(path.to_path_buf()).await.unwrap();

    app.coordinator_send_sync(UserInteractionRequired {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        interaction_type: InteractionType::SelectFromList,
        request_payload: json!({"prompt": "Select an option"}),
    })
    .await;

    let resumed_before = app.pipeline.ask(Resumed).send().await.unwrap().len();
    assert_eq!(resumed_before, 0);

    let user_response = json!({"selected": "Get Lucky"});

    app.coordinator_send_sync(InteractionResolved {
        track_id: track.id,
        plugin_name: "stage_1".to_string(),
        response_payload: user_response.clone(),
    })
    .await;

    let resumed_track = app.get_track_by_path(path.to_path_buf()).await.unwrap();
    assert_eq!(resumed_track.status, TrackStatus::Processing);

    let resumed_state = app.pipeline.ask(Resumed).send().await.unwrap();
    assert_eq!(resumed_state.len(), 1);

    let (received_track, received_payload) = &resumed_state[0];
    assert_eq!(received_track.id, track.id);
    assert_eq!(received_payload, &user_response);
}
