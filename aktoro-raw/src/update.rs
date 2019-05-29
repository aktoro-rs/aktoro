use futures_core::Stream;

use crate::actor::Actor;

pub trait Updater<A: Actor> {
    type Updated: Updated<A>;

    type Error;

    fn send(&mut self, status: A::Status) -> Result<(), Self::Error>;
}

pub trait Updated<A: Actor>: Stream<Item = A::Status> {}
