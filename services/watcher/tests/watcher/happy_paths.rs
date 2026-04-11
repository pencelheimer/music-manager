use std::fs::File;
use tempfile::TempDir;

use crate::{TestApp, WatcherEvent};

#[tokio::test]
async fn test_watcher_happy_path_new_file() {
    let temp_dir = TempDir::new().unwrap();
    let app = TestApp::new(temp_dir.path().to_path_buf(), ["mp3", "flac"]).await;

    let file_path = temp_dir.path().join("test_track.mp3");
    File::create(&file_path).unwrap();

    let events = app.wait_for_events(1).await;

    assert_eq!(events.len(), 1);

    match &events[0] {
        WatcherEvent::Ready(path) => {
            assert_eq!(path, &file_path);
        }
        _ => panic!("Expected TrackReadyToProcess event"),
    }
}

#[tokio::test]
async fn test_watcher_happy_path_remove_file() {
    let temp_dir = TempDir::new().unwrap();
    let app = TestApp::new(temp_dir.path().to_path_buf(), ["mp3"]).await;

    let file_path = temp_dir.path().join("to_be_deleted.mp3");
    std::fs::write(&file_path, "dummy data").unwrap();

    let events = app.wait_for_events(1).await;
    assert_eq!(events.len(), 1);

    std::fs::remove_file(&file_path).unwrap();

    let events = app.wait_for_events(2).await;
    assert_eq!(events.len(), 2);

    match &events[1] {
        WatcherEvent::Removed(path) => {
            assert_eq!(path, &file_path);
        }
        _ => panic!("Expected TrackRemoved event, got {:?}", events[1]),
    }
}

#[tokio::test]
async fn test_watcher_happy_path_rename_file() {
    let temp_dir = TempDir::new().unwrap();
    let app = TestApp::new(temp_dir.path().to_path_buf(), ["mp3"]).await;

    let old_path = temp_dir.path().join("old_name.mp3");
    let new_path = temp_dir.path().join("new_name.mp3");

    std::fs::write(&old_path, "dummy data").unwrap();

    let events = app.wait_for_events(1).await;
    assert_eq!(events.len(), 1);

    std::fs::rename(&old_path, &new_path).unwrap();

    let events = app.wait_for_events(3).await;
    assert_eq!(events.len(), 3);

    let recent_events = &events[1..=2];

    assert!(recent_events.contains(&WatcherEvent::Removed(old_path)));
    assert!(recent_events.contains(&WatcherEvent::Ready(new_path)));
}
