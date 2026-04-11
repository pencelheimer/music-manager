CREATE TABLE IF NOT EXISTS pending_interactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER NOT NULL UNIQUE,
    plugin_name TEXT NOT NULL,
    interaction_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,

    CONSTRAINT interaction_type_valid
    CHECK(interaction_type IN ('select-from-list', 'confirm-action', 'input-text'))
);
