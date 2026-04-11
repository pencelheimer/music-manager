use core_bin::Coordinator;
use kameo::prelude::*;

/// Sync the Coordinator mailbox
pub struct Ping;

impl Message<Ping> for Coordinator {
    type Reply = ();

    async fn handle(&mut self, _msg: Ping, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // NOTE(pencelheimer): do nothing, running this message handler means coordinator finished
        // processing other messages
    }
}
