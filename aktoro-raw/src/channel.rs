use std::error::Error as StdError;

use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::actor::Actor;
use crate::message::Handler;
use crate::message::Message;

/// The result returned by the [`Sender::try_send`]
/// method.
///
/// `Ok` contains a future resolving with the result
/// returned by the message handler.
///
/// [`Sender::try_send`]: trait.Sender.html#method.try_send
pub type SenderRes<'s, O, E> = Result<BoxFuture<'s, Result<O, E>>, E>;

pub trait Sender<A: Actor>: Clone {
    type Receiver: Receiver<A>;

    type Error: StdError + Send;

    /// Tries to send a message to be handled by the
    /// actor.
    fn try_send<M>(&mut self, msg: M) -> SenderRes<A::Output, Self::Error>
    where
        A: Handler<M>,
        M: Send;
}

pub trait Receiver<A: Actor>: Stream<Item = Box<dyn Message<Actor = A>>> {}
