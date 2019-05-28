use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::actor::Actor;
use crate::actor::Handler;
use crate::message::Message;

pub trait Sender<A: Actor>: Clone {
    type Error;

    fn send<M>(&mut self, msg: M) -> Result<BoxFuture<Result<A::Output, Self::Error>>, Self::Error>
    where
        A: Handler<M>,
        M: Send + 'static;
}

// TODO: required trait?
pub trait Receiver<A: Actor>: Stream<Item = Box<Message<Actor = A>>> {}
