use std::error;

use futures_core::Stream;

use crate::actor::Actor;
use crate::actor::Status;

pub trait Update: Status + Unpin + Send {
    fn actor_id(&self) -> u64;

    fn set_actor_id(&mut self, id: u64);
}

pub trait Updater<A: Actor>: Unpin + Send {
    type Update: Update;

    type Updated: Updated<Self::Update>;

    type Error: error::Error + Send + 'static;

    /// Tries to send an update to be handled by
    /// whatever is holding the update channel's
    /// receiver.
    fn try_send(&mut self, update: Self::Update) -> Result<(), Self::Error>;
}

pub trait Updated<U: Update>: Stream<Item = U> + Unpin + Send {}
