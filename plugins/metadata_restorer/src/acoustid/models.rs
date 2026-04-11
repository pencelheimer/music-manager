use super::dtos::Recording;
use serde::{Deserialize, Serialize};

/// A clean, flat representation of a matched track,
/// ready to be presented to the user or passed to the Coordinator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMatch {
    /// The unique MusicBrainz Identifier (MBID) for this recording.
    pub mbid: String,

    /// The title of the track.
    pub title: String,

    /// The name of the primary artist.
    pub artist: String,

    /// How closely the audio fingerprint matched the database (0.0 to 1.0).
    pub score: f64,
}

impl TrackMatch {
    /// Tries to create a TrackMatch from a raw API response.
    /// Returns None if required metadata is missing or if there is a significant duration mismatch.
    pub fn try_from_recording(rec: Recording, score: f64, expected_d: f64) -> Option<Self> {
        if rec.duration.is_some_and(|d| (d - expected_d).abs() >= 10.0) {
            return None;
        }

        let title = rec.title.unwrap_or_default();
        let artist = rec
            .artists
            .and_then(|mut artists| artists.into_iter().next())
            .map(|artist| artist.name)
            .unwrap_or_default();

        Some(Self {
            mbid: rec.id,
            title,
            artist,
            score,
        })
    }
}
