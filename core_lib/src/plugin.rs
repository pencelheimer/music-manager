pub use communication::Plugin as CommunicationPlugin;
pub use processing::Plugin as ProcessingPlugin;

mod communication {
    use std::convert::Infallible;
    use std::error::Error;
    use std::result::Result;

    use crate::state::GlobalState;

    pub trait Plugin<E>
    where
        E: Error + Send + Sync + 'static,
    {
        fn name() -> &'static str;

        fn init(state: GlobalState) -> impl Future<Output = Result<Self, E>> + Send
        where
            Self: Sized;

        fn start_loop(self) -> impl Future<Output = Result<Infallible, E>> + Send;
    }
}

mod processing {
    use std::error::Error;

    use kameo::{Actor, actor::ActorRef};

    use crate::state::GlobalState;

    pub trait Plugin<A, E>
    where
        A: Actor,
        E: Error + Send + Sync + 'static,
    {
        fn name() -> &'static str;
        fn init(state: GlobalState) -> impl Future<Output = Result<ActorRef<A>, E>> + Send;
    }
}
