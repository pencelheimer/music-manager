#![allow(unused)]

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Recording {
    pub title: String,
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub releases: Option<Vec<Release>>,
    pub genres: Option<Vec<Genre>>,
}

#[derive(Deserialize, Debug)]
pub struct ArtistCredit {
    pub name: String,
    pub joinphrase: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub status: Option<String>,
    pub media: Option<Vec<Media>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Media {
    pub track_count: Option<u32>,
    pub track_offset: Option<u32>,
    pub tracks: Option<Vec<Track>>,
}

#[derive(Deserialize, Debug)]
pub struct Track {
    pub number: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Genre {
    pub name: String,
}
