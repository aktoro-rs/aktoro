use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::action::ActionHandler;
use crate::action::ActionMessage;
use crate::actor::Actor;

pub trait Controller<A: Actor>: Clone {
    type Controlled: Controlled<A>;

    type Error;

    fn send<D>(&mut self, action: D) -> Result<BoxFuture<Result<A::Output, Self::Error>>, Self::Error>
    where
        A: ActionHandler<D>,
        D: Send + 'static;
}

pub trait Controlled<A: Actor>: Stream<Item = Box<ActionMessage<Actor = A>>> {}
