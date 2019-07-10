use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use crate::actor::Actor;

pub type AsyncMessageOutput<A> = Box<dyn Message<Actor = A>>;

pub type AsyncMessageItem<A> = Option<Box<dyn Message<Actor = A>>>;

pub trait Message: Send {
    type Actor: Actor;

    fn handle(
        &mut self,
        actor: &mut Self::Actor,
        ctx: &mut <Self::Actor as Actor>::Context,
    ) -> Result<(), <Self::Actor as Actor>::Error>;
}

pub trait AsyncMessageFut: Send {
    type Actor: Actor;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<AsyncMessageOutput<Self::Actor>>;
}

pub trait AsyncMessageStream: Send {
    type Actor: Actor;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<AsyncMessageItem<Self::Actor>>;
}

pub trait AsyncReadStream: Send {
    type Actor: Actor;

    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<AsyncMessageOutput<Self::Actor>>;
}

pub trait Handler<M: Send>: Actor {
    type Output: Send;

    /// Handles the message, returning a result
    /// eventually containing the message's output.
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Result<Self::Output, Self::Error>;
}

impl<A: Actor> Handler<()> for A {
    type Output = ();

    fn handle(&mut self, _: (), _: &mut Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }
}
