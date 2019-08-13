use std::error;

use crate::actor::Actor;

/// TODO: documentation
pub trait Handle: Unpin + Clone + Send {
    type Error: error::Error;

    /// TODO: documentation
    fn is_stopped(&self) -> bool;

    /// TODO: documentation
    fn spawn<A: Actor>(&self) -> Result<(), Self::Error>;

    /// TODO: documentation
    fn stop(&self) -> Result<(), Self::Error>;
}
