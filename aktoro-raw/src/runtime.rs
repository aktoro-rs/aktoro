use std::error::Error as StdError;
use std::future::Future;

use crate::actor::Actor;
use crate::spawned::Spawned;

pub trait Runtime {
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

    type Error: StdError;

    /// Spawns a new actor on the runtime,
    /// returning [`Some(Spawned<A>)`] if it
    /// succeeded or [`None`] if it failed or
    /// if the actor stopped itself when
    /// [`Actor::starting`] was called.
    ///
    /// [`Some(Spawned<A>)`]: sturct.Spawned.html
    /// [`Actor::starting`]: trait.Actor.html#method.starting
    fn spawn<A: Actor>(&mut self, actor: A) -> Option<Spawned<A>>;

    /// Asks to all the actors managed by the
    /// runtime, to stop, returning a future
    /// resolving after all of them have been
    /// stopped.
    fn stop(self) -> Self::Stop;

    /// Waits for all the actors to be stopped,
    /// returning a future waiting for it.
    fn wait(self) -> Self::Wait;
}
