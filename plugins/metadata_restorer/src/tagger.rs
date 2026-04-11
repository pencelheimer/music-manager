use std::{path::Path, str::FromStr};

use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, Tag, TagExt, items::Timestamp},
};
use tracing::{info, instrument, warn};

use crate::{error::TaggerError, musicbrainz::models::TrackMetadata};

pub struct AudioTagger;

impl AudioTagger {
    /// Main method to write metadata into a file
    #[instrument(skip_all, fields(path = %path.as_ref().display()))]
    pub async fn write_tags(
        &self,
        path: impl AsRef<Path>,
        metadata: &TrackMetadata,
    ) -> Result<(), TaggerError> {
        let path = path.as_ref();

        let mut tagged_file = Probe::open(path)?.read()?;

        if !tagged_file.contains_tag() {
            let tag_type = Tag::new(tagged_file.primary_tag_type());
            tagged_file.insert_tag(tag_type);
        }

        let Some(tag) = tagged_file.primary_tag_mut() else {
            return Err(TaggerError::UnsupportedFormat);
        };

        info!(
            artist = metadata.artist_display,
            title = metadata.title,
            "Writing metadata",
        );

        tag.set_title(metadata.title.clone());

        tag.set_artist(metadata.artist_display.clone());

        if let Some(album) = &metadata.album {
            tag.set_album(album.clone());
        }

        if let Some(track_num_str) = &metadata.track_number
            && let Ok(num) = track_num_str.parse::<u32>()
        {
            tag.set_track(num);
        }

        if let Some(total) = metadata.total_tracks {
            tag.set_track_total(total);
        }

        if let Some(date_str) = &metadata.release_date.clone() {
            if let Ok(timestamp) = Timestamp::from_str(date_str) {
                tag.set_date(timestamp);
            } else {
                warn!("Failed to parse date string into Timestamp: {}", date_str);
            }
        }

        // TODO(pencelheimer): fetch and set the cover art
        tag.save_to_path(path, WriteOptions::default())?;
        info!("Successfully saved tags to file");

        Ok(())
    }
}
