use std::error::Error as StdError;

use futures_core::Stream;

use crate::actor::Actor;

pub trait Updater<A: Actor> {
    type Updated: Updated<A>;

    type Error: StdError + Send;

    /// Tries to send a status update over
    /// the actor's update channel.
    fn try_send(&mut self, status: A::Status) -> Result<(), Self::Error>;
}

pub trait Updated<A: Actor>: Stream<Item = A::Status> {}
