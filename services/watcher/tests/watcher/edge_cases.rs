use std::time::Duration;

use tempfile::TempDir;

use crate::{Events, TestApp, WatcherEvent};

#[tokio::test]
async fn test_watcher_edge_ignored_extension() {
    let temp_dir = TempDir::new().unwrap();

    let app = TestApp::new(temp_dir.path().to_path_buf(), ["mp3"]).await;

    let ignored_path = temp_dir.path().join("readme.txt");
    let valid_path = temp_dir.path().join("song.mp3");

    std::fs::write(&ignored_path, "ignored data").unwrap();
    std::fs::write(&valid_path, "audio data").unwrap();

    let events = app.wait_for_events(1).await;
    assert_eq!(events.len(), 1);

    match &events[0] {
        WatcherEvent::Ready(path) => {
            assert_eq!(path, &valid_path);
        }
        _ => panic!("Expected TrackReadyToProcess event"),
    }
}

#[tokio::test]
async fn test_watcher_edge_debounce_rapid_changes() {
    let temp_dir = TempDir::new().unwrap();
    let app = TestApp::new(temp_dir.path().to_path_buf(), ["mp3"]).await;

    let file_path = temp_dir.path().join("rapid.mp3");

    for i in 0..5 {
        std::fs::write(&file_path, format!("data chunk {}", i)).unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let events = app.wait_for_events(1).await;
    assert_eq!(events.len(), 1);

    match &events[0] {
        WatcherEvent::Ready(path) => {
            assert_eq!(path, &file_path);
        }
        _ => panic!("Expected TrackReadyToProcess event"),
    }

    tokio::time::sleep(Duration::from_millis(150)).await;
    let final_events = app.coordinator.ask(Events).send().await.unwrap();

    assert_eq!(final_events.len(), 1);
}

#[tokio::test]
async fn test_watcher_edge_ghost_file() {
    let temp_dir = TempDir::new().unwrap();
    let app = TestApp::new(temp_dir.path().to_path_buf(), ["mp3"]).await;

    let ghost_path = temp_dir.path().join("ghost.mp3");
    let marker_path = temp_dir.path().join("marker.mp3");

    std::fs::write(&ghost_path, "boo").unwrap();
    std::fs::remove_file(&ghost_path).unwrap();
    std::fs::write(&marker_path, "audio data").unwrap();

    let events = app
        .wait_for_condition(|e| matches!(e, WatcherEvent::Ready(p) if p == &marker_path))
        .await;

    let ghost_is_ready = events
        .iter()
        .any(|e| matches!(e, WatcherEvent::Ready(p) if p == &ghost_path));

    assert!(!ghost_is_ready);
}
