use std::error;

use futures_core::Stream;

use crate::actor::Actor;
use crate::context::Context;
use crate::net::NetworkManager;
use crate::spawned::Spawned;

pub trait Wait<R: Runtime>: Stream<Item = Result<u64, (u64, R::Error)>> + Unpin + Send {
    /// Returns a reference to the runtime.
    fn runtime(&self) -> &R;

    /// Returns the runtime, consuming the
    /// stream.
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

    /// The type that is allowing the runtime to
    /// be polled after calling [`wait`].
    ///
    /// [`wait`]: #method.wait
    type Wait: Wait<Self>;

    type Error: error::Error + Send + 'static;

    /// Returns a list of the runtime's actors'
    /// identifier.
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

    /// Spawns a new actor on the runtime,
    /// passing its context the provided config
    /// and returning [`Some(Spawned<A>)`] if it
    /// succeeded or [`None`] if it failed or
    /// if the actor stopped itself when
    /// [`Actor::starting`] was called.
    ///
    /// [`Some(Spawned<A>)`]: sturct.Spawned.html
    /// [`Actor::starting`]: trait.Actor.html#method.starting
    fn spawn_with<A, C>(&mut self, actor: A, config: C::Config) -> Option<Spawned<A>>
    where
        A: Actor<Context = C> + 'static,
        C: Context<A>;

    /// Creates a new network manager, that
    /// can then be used by an actor to
    /// create a new TCP client, server or
    /// an UDP socket.
    fn net(&mut self) -> Self::NetworkManager;

    /// Returns a stream allowing to poll the
    /// runtime's actors.
    ///
    /// ## Note
    ///
    /// The stream can be transformed back into
    /// a runtime.
    fn wait(self) -> Self::Wait;

    /// Asks all the runtime's actors to stop.
    fn stop(&mut self);
}
