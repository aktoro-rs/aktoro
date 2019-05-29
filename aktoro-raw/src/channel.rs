use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::actor::Actor;
use crate::message::Handler;
use crate::message::Message;

pub trait Sender<A: Actor>: Clone {
    type Receiver: Receiver<A>;

    type Error;

    fn send<M>(&mut self, msg: M) -> Result<BoxFuture<Result<A::Output, Self::Error>>, Self::Error>
    where
        A: Handler<M>,
        M: Send;
}

pub trait Receiver<A: Actor>: Stream<Item = Box<Message<Actor = A>>> {}
