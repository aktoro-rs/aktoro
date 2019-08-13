use crate::actor::Actor;

use super::handled::Handled;
use super::priority::Priority;

/// TODO: documentation
pub trait Event: Send {
    /// TODO: documentation
    fn priority(&self) -> &Priority;
}

/// TODO: documentation
pub trait Handler<E: Event>: Actor {
    /// TODO: documentation
    fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<Handled<Self>, Self::Error>;
}
