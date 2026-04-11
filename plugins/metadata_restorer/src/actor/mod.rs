mod config;
mod process_track;
mod resume_processing;

use std::sync::Arc;

use async_trait::async_trait;
use config::Config;

use core_lib::{
    GlobalState, ProcessingPlugin,
    messages::{
        CoordinatorForProcessor, RegisterPlugin, StageCompleted, StageFailed,
        UserInteractionRequired,
    },
    models::{FileMetrics, InteractionType, Track},
};
use kameo::{Actor, prelude::ActorRef};
use sqlx::SqlitePool;
use tracing::{info, instrument};

use crate::{
    acoustid::{AcoustIdClient, TrackMatch},
    db::fingerprint,
    error::PluginError,
    misc::UserAgent,
    musicbrainz::MusicBrainzClient,
    tagger::AudioTagger,
};

/// The MetadataRestorer plugin responsible for restoring the matadata of provided files.
///
/// It calculates the chromaprint fingerprint of the tracks, fetches AcoustID
/// matches for that fingerprint, and restores metadata form the matched
/// MusicBrainz page.
pub struct MetadataRestorer<C: CoordinatorForProcessor> {
    aid_client: AcoustIdClient,
    mb_client: MusicBrainzClient,
    tagger: AudioTagger,

    /// The actor where processed events are sent.
    target_actor: ActorRef<C>,
    db_pool: SqlitePool,
}

impl<C: CoordinatorForProcessor> MetadataRestorer<C> {
    pub async fn send_stage_failed(
        &self,
        track_id: i64,
        err: impl ToString,
    ) -> Result<(), PluginError> {
        let plugin_name = Self::plugin_name().to_string();

        self.target_actor
            .tell(StageFailed {
                track_id,
                plugin_name,
                error_message: err.to_string(),
            })
            .await
            .map_err(|e| PluginError::coordinator_dead(e.to_string()))?;

        Ok(())
    }

    pub async fn send_stage_completed(
        &self,
        track_id: i64,
        new_metrics: Option<FileMetrics>,
    ) -> Result<(), PluginError> {
        let plugin_name = Self::plugin_name().to_string();

        self.target_actor
            .tell(StageCompleted {
                track_id,
                plugin_name,
                new_metrics,
                new_path: None,
            })
            .await
            .map_err(|e| PluginError::coordinator_dead(e.to_string()))?;

        Ok(())
    }

    pub async fn send_stage_paused(
        &self,
        track_id: i64,
        request_payload: serde_json::Value,
    ) -> Result<(), PluginError> {
        let plugin_name = Self::plugin_name().to_string();
        let interaction_type = InteractionType::SelectFromList;

        self.target_actor
            .tell(UserInteractionRequired {
                track_id,
                plugin_name,
                interaction_type,
                request_payload,
            })
            .await
            .map_err(|e| PluginError::coordinator_dead(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip_all, fields(mbid = candidate.mbid))]
    async fn handle_candidate(
        &self,
        track: &Track,
        candidate: &TrackMatch,
    ) -> Result<(), PluginError> {
        fingerprint::set_mbid(&self.db_pool, track.id, &candidate.mbid).await?;

        let metadata = self.mb_client.fetch_metadata(&candidate.mbid).await?;

        self.tagger.write_tags(&track.file_path, &metadata).await?;

        Ok(())
    }
}

/// Arguments required to initialize and spawn the WatcherService.
pub struct MetadataRestorerArgs<C: CoordinatorForProcessor> {
    /// Config of the actor.
    pub config: Config,

    /// Global DB Pool
    pub db_pool: SqlitePool,

    /// The coordinator or handler actor that will receive the resulting events.
    pub target_actor: ActorRef<C>,
}

impl<C: CoordinatorForProcessor> MetadataRestorerArgs<C> {
    pub fn with_state(state: &GlobalState, target_actor: ActorRef<C>) -> Result<Self, PluginError> {
        let config = Config::new(state)?;
        let db_pool = state.db_pool.clone();

        Ok(Self {
            config,
            db_pool,
            target_actor,
        })
    }

    pub fn new(
        db_pool: SqlitePool,
        acoustid_api_url: String,
        acoustid_api_key: String,
        musicbrainz_api_url: String,
        similarity_threshold: f64,
        user_agent: UserAgent,
        target_actor: ActorRef<C>,
    ) -> Self {
        let config = Config {
            acoustid_api_url,
            acoustid_api_key,
            musicbrainz_api_url,
            similarity_threshold,
            user_agent,
        };

        Self {
            config,
            db_pool,
            target_actor,
        }
    }
}

impl<C: CoordinatorForProcessor> Actor for MetadataRestorer<C> {
    type Args = MetadataRestorerArgs<C>;
    type Error = PluginError;

    /// Initializes the actor and configures Lua settings.
    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        info!("Starting Metadata Restorer");

        let Config {
            acoustid_api_url,
            acoustid_api_key,
            musicbrainz_api_url,
            similarity_threshold,
            user_agent,
        } = args.config;

        let acoustid_client = AcoustIdClient::new(
            acoustid_api_url,
            acoustid_api_key,
            similarity_threshold,
            user_agent.clone(),
        )?;

        let musicbrainz_client = MusicBrainzClient::new(musicbrainz_api_url, user_agent)?;

        args.target_actor
            .tell(RegisterPlugin {
                name: Self::plugin_name().to_string(),
                processor: Arc::new(actor_ref),
            })
            .await
            .map_err(PluginError::coordinator_dead)?;

        Ok(Self {
            aid_client: acoustid_client,
            mb_client: musicbrainz_client,
            tagger: AudioTagger,
            target_actor: args.target_actor,
            db_pool: args.db_pool,
        })
    }
}

#[async_trait]
impl<C: CoordinatorForProcessor> ProcessingPlugin for MetadataRestorer<C> {
    fn plugin_name() -> &'static str {
        "metadata-restorer"
    }
}
