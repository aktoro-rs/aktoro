use std::error::Error as StdError;
use std::future::Future;

use crate::actor::Actor;
use crate::net::NetworkManager;
use crate::spawned::Spawned;

pub trait Runtime {
    /// The type that is handling the types of
    /// the TCP socket client and server and
    /// of the UDP socket that actors can use
    /// to be compatible with the runtime (this
    /// might not be necessary depending on the
    /// runtime implementation).
    type NetworkManager: NetworkManager;

    /// The future returned after calling the
    /// [`stop`] method. It will resolve after
    /// all the actors have been stopped.
    ///
    /// [`stop`]: #method.stop
    type Stop: Future<Output = Result<(), Self::Error>>;

    /// The future returned after calling the
    /// [`wait`] method. It will resolve after
    /// all the actors have been stopped.
    ///
    /// [`wait`]: #method.wait
    type Wait: Future<Output = Result<(), Self::Error>>;

    type Error: StdError + Send;

    /// Spawns a new actor on the runtime,
    /// returning [`Some(Spawned<A>)`] if it
    /// succeeded or [`None`] if it failed or
    /// if the actor stopped itself when
    /// [`Actor::starting`] was called.
    ///
    /// [`Some(Spawned<A>)`]: sturct.Spawned.html
    /// [`Actor::starting`]: trait.Actor.html#method.starting
    fn spawn<A: Actor>(&mut self, actor: A) -> Option<Spawned<A>>;

    /// Creates a new network manager, that
    /// can then be used by an actor to
    /// create a new TCP client, server or
    /// an UDP socket.
    fn net(&mut self) -> Self::NetworkManager;

    /// Asks to all the actors managed by the
    /// runtime to stop, returning a future
    /// resolving after all of them have been
    /// stopped.
    fn stop(self) -> Self::Stop;

    /// Waits for all the actors to be stopped,
    /// returning a future waiting for it.
    fn wait(self) -> Self::Wait;
}
