mod acoustid;
mod actor;
mod chromaprint;
mod db;
mod error;
mod misc;
mod musicbrainz;
mod tagger;

pub use actor::{MetadataRestorer, MetadataRestorerArgs};
pub use error::PluginError as MetadataRestorerError;
