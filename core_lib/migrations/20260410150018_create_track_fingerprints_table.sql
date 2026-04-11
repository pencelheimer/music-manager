CREATE TABLE track_fingerprints (
    track_id INTEGER PRIMARY KEY REFERENCES tracks(id),
    chromaprint TEXT NOT NULL,
    duration REAL NOT NULL,
    mbid TEXT,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
