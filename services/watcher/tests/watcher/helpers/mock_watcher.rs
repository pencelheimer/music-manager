use core_lib::messages::CoordinatorForService;
use kameo::prelude::*;
use watcher::WatcherService;

/// Sync the Watcher mailbox
pub struct Ping;

impl<C: CoordinatorForService> Message<Ping> for WatcherService<C> {
    type Reply = ();

    async fn handle(&mut self, _msg: Ping, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // NOTE(pencelheimer): do nothing, running this message handler means Watcher is ready
    }
}
