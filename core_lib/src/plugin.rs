pub use processing::Plugin as ProcessingPlugin;
pub use service::Plugin as ServicePlugin;

mod service {
    use crate::state::GlobalState;
    use async_trait::async_trait;
    use kameo::{Actor, actor::ActorRef};

    /// A trait for background services and event generators.
    ///
    /// Service plugins are long-running tasks that run independently in the background,
    /// such as file system watchers, external API listeners, servers, etc.
    #[async_trait]
    pub trait Plugin: Actor + Sized {
        /// Returns the unique identifier/name of the service plugin.
        fn plugin_name() -> &'static str;

        /// Initializes the service with the global application state and starts it.
        ///
        /// Returns an `ActorRef` that the coordinator or other components can use
        /// to send messages to spawned actors.
        async fn init(state: GlobalState) -> Result<ActorRef<Self>, Self::Error>;
    }
}

mod processing {
    use crate::{messages::TrackProcessor, state::GlobalState};
    use async_trait::async_trait;
    use kameo::{Actor, actor::ActorRef};
    use std::sync::Arc;

    /// A trait for on-demand processing plugins.
    ///
    /// Processing plugins are actor-based components that handle specific, finite
    /// tasks like audio transcoding or metadata extraction. They are initialized
    /// with the shared global state and return an `ActorRef` for message passing.
    #[async_trait]
    pub trait Plugin: Actor + Sized {
        /// Returns the unique identifier/name of the processing plugin.
        fn plugin_name() -> &'static str;

        /// Initializes the plugin with the global application state and starts it.
        ///
        /// Returns an `ActorRef` that the coordinator or other components can use
        /// to send messages to spawned actors.
        async fn init(state: GlobalState) -> Result<ActorRef<Self>, Self::Error>;

        /// Helper for building the Coordinator's plugin registry.
        ///
        /// Initializes the plugin and converts its strictly typed ActorRef
        /// into a dynamically dispatched Arc<dyn TrackProcessor>.
        async fn init_registered(
            state: GlobalState,
        ) -> Result<(String, Arc<dyn TrackProcessor>), Self::Error>
        where
            ActorRef<Self>: TrackProcessor + 'static,
        {
            let actor_ref = Self::init(state).await?;
            let erased: Arc<dyn TrackProcessor> = Arc::new(actor_ref);
            Ok((Self::plugin_name().to_string(), erased))
        }
    }
}
