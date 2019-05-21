use futures_core::future::BoxFuture;

use crate::actor::Actor;
use crate::actor::Handler;

pub trait Sender<A: Actor> {
    fn send<M>(&mut self, msg: M) -> BoxFuture<<A as Handler<M>>::Output>
    where
        A: Handler<M>;
}

pub trait Receiver<A: Actor> {
    type Sender: Sender<A>;

    fn try_recv<M>(&mut self) -> Option<(M, &mut Respond<A, M>)>
    where
        A: Handler<M>;
}

pub trait Respond<A: Actor, M>
where
    A: Handler<M>
{
    fn respond(&mut self, resp: A::Output);
}
