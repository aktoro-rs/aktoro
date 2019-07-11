use std::error::Error as StdError;

use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::action::Action;
use crate::action::ActionHandler;
use crate::actor::Actor;

/// The result returned by the [`Controller::try_send`]
/// method.
///
/// `Ok` contains a future resolving with the result
/// returned by the action handler.
///
/// [`Controller::try_send`]: trait.Controller.html#method.try_send
pub type ControllerRes<'c, O, E> = Result<BoxFuture<'c, Result<O, E>>, E>;

pub trait Controller<A: Actor>: Unpin + Clone + Send {
    type Controlled: Controlled<A>;

    type Error: StdError + Send + 'static;

    /// Tries to send an action to be handled by the
    /// actor.
    fn try_send<D>(&mut self, action: D) -> ControllerRes<A::Output, Self::Error>
    where
        A: ActionHandler<D>,
        D: Send + 'static;
}

pub trait Controlled<A: Actor>: Stream<Item = Box<dyn Action<Actor = A>>> + Unpin + Send {}
