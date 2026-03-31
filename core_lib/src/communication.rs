use std::convert::Infallible;
use std::error::Error;
use std::result::Result;

use crate::state::GlobalState;

pub trait CommunicationPlugin<E>
where
    E: Error + Send + Sync + 'static,
{
    fn name() -> &'static str;

    async fn init(state: GlobalState) -> Result<Self, E>
    where
        Self: Sized;

    async fn start_loop(self) -> Result<Infallible, E>;
}
