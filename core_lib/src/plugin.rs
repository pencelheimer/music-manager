pub use processing::Plugin as ProcessingPlugin;
pub use service::Plugin as ServicePlugin;

mod service {
    use std::convert::Infallible;
    use std::error::Error;

    /// A trait for background services and event generators.
    ///
    /// Service plugins are long-running tasks that run independently in the background,
    /// such as file system watchers, external API listeners, servers, etc.
    pub trait Plugin<E>
    where
        E: Error + Send + Sync + 'static,
    {
        /// Returns the unique identifier/name of the service plugin.
        fn name() -> &'static str;

        /// Starts the infinite background loop of the service.
        ///
        /// This method is expected to run indefinitely. If it ever returns,
        /// it indicates a fatal error within the service's execution.
        fn start_loop(self) -> impl Future<Output = Result<Infallible, E>> + Send;
    }
}

mod processing {
    use std::error::Error;

    use kameo::{Actor, actor::ActorRef};

    use crate::state::GlobalState;

    /// A trait for on-demand processing plugins.
    ///
    /// Processing plugins are actor-based components that handle specific, finite
    /// tasks like audio transcoding or metadata extraction. They are initialized
    /// with the shared global state and return an `ActorRef` for message passing.
    pub trait Plugin<A, E>
    where
        A: Actor,
        E: Error + Send + Sync + 'static,
    {
        /// Returns the unique identifier/name of the processing plugin.
        fn name() -> &'static str;

        /// Initializes the plugin with the global application state and starts it.
        ///
        /// Returns an `ActorRef` that the coordinator or other components can use
        /// to send messages to spawned actors.
        fn init(state: GlobalState) -> impl Future<Output = Result<ActorRef<A>, E>> + Send;
    }
}
