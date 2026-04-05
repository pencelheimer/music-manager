use core_lib::{
    messages::{StageCompleted, StageFailed, UserInteractionRequired},
    models::TrackStatus,
};
use kameo::prelude::{Context, Message};
use tracing::{error, info, instrument, warn};

use crate::{
    Coordinator,
    db::{interactions, tracks},
};

impl Message<StageCompleted> for Coordinator {
    type Reply = ();

    #[instrument(
        skip_all,
        fields(
            track_id = msg.track_id,
            stage = %msg.plugin_name
        )
    )]
    async fn handle(
        &mut self,
        msg: StageCompleted,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("Plugin successfully completed its stage");

        let id = msg.track_id;
        let path = msg.new_path.as_deref();
        let metrics = msg.new_metrics.as_ref();

        if (path.is_some() || metrics.is_some())
            && let Err(e) = tracks::apply_modifications(&self.db_pool, id, path, metrics).await
        {
            error!(error = %e, "Failed to save track modifications from plugin");
            return;
        }

        self.advance_pipeline(id).await;
    }
}

impl Message<StageFailed> for Coordinator {
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
        msg: StageFailed,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        warn!(error = %msg.error_message, "Plugin failed to process track");

        if let Err(e) = tracks::set_status(&self.db_pool, msg.track_id, TrackStatus::Failed).await {
            error!(error = %e, "Failed to update track status to Failed");
        }
    }
}

impl Message<UserInteractionRequired> for Coordinator {
    type Reply = ();

    #[instrument(
        skip_all,
        fields(
            track_id = msg.track_id,
            plugin = %msg.plugin_name,
            interaction = ?msg.interaction_type
        )
    )]
    async fn handle(
        &mut self,
        msg: UserInteractionRequired,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        info!("Plugin requested user interaction. Pausing track processing.");

        let UserInteractionRequired {
            track_id: id,
            plugin_name: name,
            interaction_type: itype,
            request_payload: req,
        } = msg;

        if let Err(e) = interactions::pause_processing(&self.db_pool, id, &name, itype, req).await {
            error!(error = %e, "Failed to pause processing pipeline and save interaction request");
            return;
        }

        // TODO(pencelheimer): maybe broadcast an event for reactive UI changes
    }
}
