use core_lib::{messages::InteractionResolved, models::TrackStatus};
use kameo::prelude::{Context, Message};
use tracing::{error, info, instrument};

use crate::{Coordinator, db::interactions};

impl Message<InteractionResolved> for Coordinator {
    type Reply = ();

    #[instrument(
        skip_all,
        fields(
            track_id = msg.track_id,
            plugin = %msg.plugin_name
        )
    )]
    async fn handle(
        &mut self,
        msg: InteractionResolved,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("Received user response. Resuming track processing.");

        let InteractionResolved {
            track_id,
            plugin_name,
            response_payload,
        } = msg;

        let Some(track) = self.get_track(track_id).await else {
            return;
        };

        if track.status != TrackStatus::PausedWaitingUser {
            tracing::warn!(
                status = ?track.status,
                "Received InteractionResolved for a track that is not waiting for user. Ignoring."
            );
            return;
        }

        if let Err(e) = interactions::resolve_interaction(&self.db_pool, track_id).await {
            error!(error = %e, "Failed to resolve interaction in database");
            return;
        }

        self.with_plugin(&plugin_name, track.id, |plugin| async move {
            plugin.send_resume_processing(track, response_payload).await
        })
        .await;
    }
}
