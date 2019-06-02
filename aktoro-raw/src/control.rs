use std::error::Error as StdError;

use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::action::Action;
use crate::action::ActionHandler;
use crate::actor::Actor;

pub type ControllerRes<'c, O, E> = Result<BoxFuture<'c, Result<O, E>>, E>;

pub trait Controller<A: Actor>: Clone {
    type Controlled: Controlled<A>;

    type Error: StdError + Send;

    fn try_send<D>(&mut self, action: D) -> ControllerRes<A::Output, Self::Error>
    where
        A: ActionHandler<D>,
        D: Send + 'static;
}

pub trait Controlled<A: Actor>: Stream<Item = Box<dyn Action<Actor = A>>> {}
