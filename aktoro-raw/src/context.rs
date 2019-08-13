use std::error;

use crate::actor::Actor;
use crate::actor::Handle;
use crate::handler::action;
use crate::handler::action::Action;
use crate::handler::event;
use crate::handler::event::Event;

/// TODO: documentation
///
/// TODO(method): exec future; eventually blocking io-wise; eventually waiting for it
/// TODO(method): subscribe to stream or reader
/// TODO(trait): convert writing to async writer to an executable future
pub trait Context<A>: Unpin + Send + Sized
where
    A: Actor<Context = Self>,
{
    /// TODO: documentation
    ///
    /// TODO(trait): eventually a `ContextConfig` trait (requires a `RuntimeConfig` one)
    type Config: Default;

    /// TODO: documentation
    type Handle: Handle<A>;

    type Error: error::Error;

    /// TODO: documentation
    fn new(actor_id: u64, config: Self::Config) -> Result<Self, Self::Error>;

    /// TODO: documentation
    fn status(&self) -> &A::Status;

    /// TODO: documentation
    ///
    /// TODO(return): an handle to the result
    fn exec<D>(&self, action: D) -> Result<(), Self::Error>
    where
        A: action::Handler<D>,
        D: Action;

    /// TODO: documentation
    fn emit<E>(&self, event: E) -> Result<(), Self::Error>
    where
        A: event::Handler<E>,
        E: Event;

    /// TODO: documentation
    fn handle(&self) -> Result<Self::Handle, Self::Error>;

    /// TODO: documentation
    ///
    /// TODO(param): link type
    fn link<H, C>(&self, handle: &H) -> Result<(), Self::Error>
    where
        H: Handle<C>,
        C: Actor;
}
