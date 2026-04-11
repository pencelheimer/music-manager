#![allow(unused)]

use super::dtos::{ArtistCredit, Genre, Recording, Release};

/// Track metadata
#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub title: String,

    /// e.g. "Gorillaz feat. De La Soul"
    pub artist_display: String,

    /// e.g. ["Gorillaz", "De La Soul"]
    pub artists: Vec<String>,

    pub album: Option<String>,
    pub release_date: Option<String>,
    pub cover_art_url: Option<String>,

    pub genres: Vec<String>,

    pub track_number: Option<String>,
    pub total_tracks: Option<u32>,
    // TODO(pencelheimer): compositors via work-rels
}

#[derive(Default)]
struct TrackMetadataBuilder {
    title: String,
    artist_display: String,
    artists: Vec<String>,
    album: Option<String>,
    genres: Vec<String>,
    release_date: Option<String>,
    track_number: Option<String>,
    total_tracks: Option<u32>,
    cover_art_id: Option<String>,
}

impl TrackMetadataBuilder {
    fn new(title: String) -> Self {
        Self {
            title,
            ..Default::default()
        }
    }

    fn parse_artists(mut self, credits: Option<Vec<ArtistCredit>>) -> Self {
        if let Some(credits) = credits {
            for credit in credits {
                self.artists.push(credit.name.clone());
                self.artist_display.push_str(&credit.name);
                self.artist_display
                    .push_str(credit.joinphrase.as_deref().unwrap_or(""));
            }
        }

        if self.artists.is_empty() {
            self.artist_display = String::from("Unknown Artist");
        }

        self
    }

    fn parse_genres(mut self, genres: Option<Vec<Genre>>) -> Self {
        self.genres = genres
            .unwrap_or_default()
            .into_iter()
            .map(|g| g.name)
            .collect();

        self
    }

    fn parse_releases(mut self, releases: Option<Vec<Release>>) -> Self {
        let best_release = releases
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.status.as_deref() == Some("Official") && r.date.is_some())
            .min_by_key(|r| r.date.clone());

        let Some(release) = best_release else {
            return self;
        };

        self.album = Some(release.title);
        self.release_date = release.date;
        self.cover_art_id = Some(release.id);

        if let Some(media) = release.media.and_then(|m| m.into_iter().next()) {
            self.total_tracks = media.track_count;
            if let Some(track) = media.tracks.and_then(|t| t.into_iter().next()) {
                self.track_number = track.number;
            }
        }

        self
    }

    fn build(self) -> TrackMetadata {
        TrackMetadata {
            title: self.title,
            artist_display: self.artist_display,
            artists: self.artists,
            album: self.album,
            genres: self.genres,
            release_date: self.release_date,
            track_number: self.track_number,
            total_tracks: self.total_tracks,
            cover_art_url: self.cover_art_id,
        }
    }
}

impl From<Recording> for TrackMetadata {
    fn from(rec: Recording) -> Self {
        TrackMetadataBuilder::new(rec.title)
            .parse_artists(rec.artist_credit)
            .parse_genres(rec.genres)
            .parse_releases(rec.releases)
            .build()
    }
}
