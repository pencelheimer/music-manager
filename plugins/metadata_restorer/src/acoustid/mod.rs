#![allow(unused)]

mod dtos;
mod models;

use dtos::AcoustIdResponse;
pub use models::TrackMatch;

use std::time::Duration;

use itertools::Itertools as _;
use reqwest::Client;
use tracing::instrument;

use crate::{chromaprint::Fingerprint, error::AcoustIdError, misc::UserAgent};

pub struct AcoustIdClient {
    client: Client,
    api_url: String,
    api_key: String,
    similarity_threshold: f64,
}

impl AcoustIdClient {
    pub fn new(
        api_url: impl ToString,
        api_key: impl ToString,
        similarity_threshold: f64,
        user_agent: UserAgent,
    ) -> Result<Self, AcoustIdError> {
        let client = Client::builder()
            .user_agent(&*user_agent)
            .timeout(Duration::from_secs(10)) // TODO(pencelheimer): set from config
            .build()?;

        Ok(Self {
            client,
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            similarity_threshold,
        })
    }

    pub fn with_url(mut self, url: impl ToString) -> Self {
        self.api_url = url.to_string();
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    async fn request(&self, fp: &Fingerprint) -> Result<AcoustIdResponse, AcoustIdError> {
        Ok(self
            .client
            .get(&self.api_url)
            .query(&[
                ("client", self.api_key.as_str()),
                ("meta", "recordings compress"),
                ("duration", &fp.duration.round().to_string()),
                ("fingerprint", &fp.fingerprint),
            ])
            .send()
            .await?
            .json::<AcoustIdResponse>()
            .await?)
    }

    /// Searches for the MBID by the Fingerprint
    #[instrument(
        skip_all,
        fields(
            duration = fp.duration,
            fingerprint = %fp.fingerprint[..fp.fingerprint.len().min(10)]
        )
    )]
    pub async fn lookup(&self, fp: &Fingerprint) -> Result<Vec<TrackMatch>, AcoustIdError> {
        let results = self.request(fp).await?.results()?;

        let matches = results
            .into_iter()
            .filter(|r| r.score > self.similarity_threshold)
            .flat_map(|r| {
                r.recordings.into_iter().flatten().filter_map(move |rec| {
                    TrackMatch::try_from_recording(rec, r.score, fp.duration)
                })
            })
            .unique_by(|m| {
                if m.title.is_empty() && m.artist.is_empty() {
                    m.mbid.clone()
                } else {
                    format!("{}|{}", m.title.to_lowercase(), m.artist.to_lowercase())
                }
            })
            .sorted_by(|a, b| b.score.total_cmp(&a.score))
            .collect();

        Ok(matches)
    }
}
