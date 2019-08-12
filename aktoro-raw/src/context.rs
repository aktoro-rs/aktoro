use std::error;

use crate::actor::Actor;

/// TODO: documentation
///
/// TODO(method): do action
/// TODO(method): emit event
/// TODO(method): spawn an actor
/// TODO(method): exec future; eventually blocking io-wise; eventually waiting for it
/// TODO(method): subscribe to stream or reader
/// TODO(trait): convert writing to async writer to an executable future
pub trait Context<A: Actor>: Unpin + Send + Sized {
    /// TODO: documentation
    ///
    /// TODO(trait): eventually a `ContextConfig` trait (requires a `RuntimeConfig` one)
    type Config: Default;

    type Error: error::Error;

    /// TODO: documentation
    fn new(actor_id: u64, config: Self::Config) -> Result<Self, Self::Error>;

    /// TODO: documentation
    fn status(&self) -> &A::Status;
}
