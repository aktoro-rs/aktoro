use futures_core::future::BoxFuture;
use futures_core::Stream;

use crate::actor::Actor;

pub trait Controller<A: Actor>: Clone {
    type Error;

    fn send(
        &mut self,
        action: A::Action,
    ) -> Result<BoxFuture<Result<A::Status, Self::Error>>, Self::Error>;
}

// TODO: required trait?
pub trait Controlled<A: Actor>: Stream<Item = Box<Update<A>>> {}

pub trait Update<A: Actor>: Send {
    fn action(&self) -> &A::Action;

    fn update(&mut self, status: A::Status); // TODO: Result?
}
