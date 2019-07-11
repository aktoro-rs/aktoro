use std::error::Error as StdError;

use futures_core::Stream;

use crate::actor::Actor;
use crate::net::NetworkManager;
use crate::spawned::Spawned;

pub trait Wait<R: Runtime>: Stream<Item = Result<u64, (u64, R::Error)>> + Unpin + Send {
    fn runtime(&self) -> &R;

    fn into_runtime(self) -> R;
}

pub trait Runtime: Default + Unpin + Send {
    /// The type that is handling the types of
    /// the TCP socket client and server and
    /// of the UDP socket that actors can use
    /// to be compatible with the runtime (this
    /// might not be necessary depending on the
    /// runtime implementation).
    type NetworkManager: NetworkManager;

    // TODO
    type Wait: Wait<Self>;

    type Error: StdError + Send + 'static;

    // TODO
    fn actors(&self) -> Vec<u64>;

    /// Spawns a new actor on the runtime,
    /// returning [`Some(Spawned<A>)`] if it
    /// succeeded or [`None`] if it failed or
    /// if the actor stopped itself when
    /// [`Actor::starting`] was called.
    ///
    /// [`Some(Spawned<A>)`]: sturct.Spawned.html
    /// [`Actor::starting`]: trait.Actor.html#method.starting
    fn spawn<A>(&mut self, actor: A) -> Option<Spawned<A>>
    where
        A: Actor + 'static;

    /// Creates a new network manager, that
    /// can then be used by an actor to
    /// create a new TCP client, server or
    /// an UDP socket.
    fn net(&mut self) -> Self::NetworkManager;

    // TODO
    fn wait(self) -> Self::Wait;

    // TODO
    fn stop(&mut self);
}
