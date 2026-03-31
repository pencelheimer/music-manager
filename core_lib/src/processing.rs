use std::error::Error;

use kameo::{Actor, actor::ActorRef};

use crate::state::GlobalState;

pub trait ProcessingPlugin<A, E>
where
    A: Actor,
    E: Error + Send + Sync + 'static,
{
    fn name() -> &'static str;
    async fn init(state: GlobalState) -> Result<ActorRef<A>, E>;
}
