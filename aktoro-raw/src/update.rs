use std::error::Error as StdError;

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

    type Error: StdError + Send + 'static;

    // TODO
    fn try_send(&mut self, update: Self::Update) -> Result<(), Self::Error>;
}

pub trait Updated<U: Update>: Stream<Item = U> + Unpin + Send {}
