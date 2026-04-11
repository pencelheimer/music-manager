#![allow(unused)]

use serde::Deserialize;

use crate::error::AcoustIdError;

/// Response of the AcoustID API
#[derive(Debug, Deserialize)]
pub(crate) struct AcoustIdResponse {
    /// Status of the response ("ok" or "error")
    status: String,

    /// Possible error details (present if status is "error")
    error: Option<AcoustIdErrorDetail>,

    /// Search results (present if status is "ok")
    results: Option<Vec<AcoustIdResult>>,
}

/// Error details returned by the AcoustID
#[derive(Debug, Deserialize)]
pub(crate) struct AcoustIdErrorDetail {
    /// Error message
    message: String,
}

/// AcoustID match result
#[derive(Debug, Deserialize)]
pub(crate) struct AcoustIdResult {
    /// Similarity score
    pub score: f64,

    /// List of matched recordings
    pub recordings: Option<Vec<Recording>>,
}

/// Struct representing a recording from MusicBrainz (via AcoustID)
#[derive(Debug, Deserialize)]
pub(crate) struct Recording {
    /// MusicBrainz ID of the track
    pub id: String,

    /// Title of the track (requires `meta=recordings compress` query param)
    pub title: Option<String>,

    /// List of artists associated with the track
    pub artists: Option<Vec<Artist>>,

    /// Duration of the track
    pub duration: Option<f64>,
}

/// Struct representing an artist
#[derive(Debug, Deserialize)]
pub(crate) struct Artist {
    /// Name of the artist
    pub name: String,
}

impl AcoustIdResponse {
    /// Extracts the results or returns an API error if the request failed on the server side.
    pub(crate) fn results(self) -> Result<Vec<AcoustIdResult>, AcoustIdError> {
        self.results.ok_or_else(|| {
            AcoustIdError::api_error(
                self.error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Unknown API error".to_string()),
            )
        })
    }
}
