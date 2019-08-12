use crate::actor::Actor;

use super::handled::Handled;
use super::priority::Priority;

/// TODO: documentation
pub trait Message: Send {
    fn priority(&self) -> Priority {
        Priority::new()
    }
}

/// TODO: documentation
pub trait Handler<M: Message>: Actor {
    /// TODO: documentation
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Result<Handled<Self>, Self::Error>;
}
