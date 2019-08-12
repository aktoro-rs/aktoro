use crate::actor::Actor;

use super::handled::Output;
use super::priority::Priority;

/// TODO: documentation
pub trait Action: Send {
    /// TODO: documentation
    fn priority(&self) -> Priority {
        Priority::new()
    }
}

/// TODO: documentation
pub trait Handler<A: Action>: Actor {
    /// TODO: documentation
    type Output: Unpin + Send;

    /// TODO: documentation
    fn handle(&mut self, action: A, ctx: &mut Self::Context) -> Result<Output<Self, Self::Output>, Self::Error>;
}
