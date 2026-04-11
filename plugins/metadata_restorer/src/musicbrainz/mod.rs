pub(crate) mod dtos;
pub(crate) mod models;

use dtos::Recording;
use models::TrackMetadata;

use reqwest::Client;
use std::time::Duration;
use tracing::instrument;

use crate::{error::MusicBrainzError, misc::UserAgent};

pub struct MusicBrainzClient {
    client: Client,
    api_url: String,
}

impl MusicBrainzClient {
    pub fn new(api_url: String, user_agent: UserAgent) -> Result<Self, MusicBrainzError> {
        let client = Client::builder()
            .user_agent(&*user_agent)
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client,
            api_url: api_url.to_string(),
        })
    }

    async fn request(&self, mbid: impl AsRef<str>) -> Result<Recording, MusicBrainzError> {
        let response = self
            .client
            .get(format!("{}/recording/{}", self.api_url, mbid.as_ref()))
            .query(&[("inc", "artists+releases+genres+media"), ("fmt", "json")])
            .send()
            .await?;

        if response.status() == 404 {
            return Err(MusicBrainzError::not_found(mbid.as_ref().to_string()));
        }

        let recording: Recording = response.json().await?;

        Ok(recording)
    }

    /// Loads full metadata by recording MBID
    #[instrument(skip(self), fields(mbid = %mbid.as_ref()))]
    pub async fn fetch_metadata(
        &self,
        mbid: impl AsRef<str>,
    ) -> Result<TrackMetadata, MusicBrainzError> {
        let response = self.request(mbid).await?;
        Ok(TrackMetadata::from(response))
    }
}
