CREATE TABLE IF NOT EXISTS tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,

    mtime DATETIME NOT NULL,
    file_size INTEGER NOT NULL,

    current_stage TEXT NOT NULL DEFAULT 'init',
    status TEXT NOT NULL DEFAULT 'pending',

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT status_valid
    CHECK(status IN ('pending', 'processing', 'paused-waiting-user', 'completed', 'failed', 'missing'))
);

CREATE INDEX IF NOT EXISTS idx_tracks_file_path ON tracks(file_path);
CREATE INDEX IF NOT EXISTS idx_tracks_identity ON tracks(file_size, mtime);

CREATE TRIGGER IF NOT EXISTS tracks_updated_at_trigger
AFTER UPDATE ON tracks
FOR EACH ROW
WHEN NEW.updated_at < OLD.updated_at
BEGIN
    UPDATE tracks
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;
