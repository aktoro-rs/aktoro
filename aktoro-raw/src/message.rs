use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use futures_io::Error as FutIOError;

use crate::actor::Actor;

pub type AsyncMessageFutOutput = Box<dyn Message<Actor = Self::Actor>>;

pub type AsyncMessageStreamItem = Option<Box<dyn Message<Actor = Self::Actor>>>;

pub type AsyncReadStreamItem = Result<Box<dyn Message<Actor = Self::Actor>>, FutIOError>;

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
    ) -> Poll<AsyncMessageFutOutput>;
}

pub trait AsyncMessageStream: Send {
    type Actor: Actor;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<AsyncMessageStreamItem>;
}

pub trait AsyncReadStream: Send {
    type Actor: Actor;

    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<AsyncReadStreamItem>;
}

pub trait Handler<M: Send>: Actor {
    type Output: Send;

    /// Handles the message, returning a result
    /// eventually containing the message's output.
    fn handle(&mut self, msg: M, ctx: &mut Self::Context) -> Result<Self::Output, Self::Error>;
}
