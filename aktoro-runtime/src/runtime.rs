use std::future::Future;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_raw as raw;
use aktoro_raw::Context as RawContext;
use fnv::FnvHashMap;
use futures_core::Stream;
use rand::FromEntropy;
use rand::RngCore;
use rand_xoshiro::Xoshiro512StarStar;

use crate::actor;
use crate::actor::Actor;
use crate::actor::KillSender as Kill;
use crate::actor::KilledRecver;
use crate::actor::KilledSender;
use crate::error::Error;

/// An actor runtime using the [`runtime`] crate.
///
/// [`runtime`]: https://docs.rs/runtime
pub struct Runtime {
    /// A map matching an actor's ID with
    /// a sender for its kill channel.
    actors: FnvHashMap<u64, Kill>,
    /// A sender for the actors' killed
    /// channel (it will be cloned and
    /// passed to all new actors).
    sender: KilledSender,
    /// A receiver the the actors' killed
    /// channel, notified when an actor
    /// has stopped/been killed.
    recver: KilledRecver,
    /// A fast (non-cryptographic) random
    /// number generator.
    rng: Xoshiro512StarStar,
}

/// A future that resolves when all the
/// runtime's actors have been stopped.
pub struct Stop(Wait);

/// A future that resolves when all the
/// runtime's actors have been stopped.
pub struct Wait {
    rt: Runtime,
    /// Contains a list of all the errors
    /// that happened while waiting for
    /// the actors to stop.
    errors: Vec<Error>,
}

impl Runtime {
    /// Creates a new `Runtime`.
    pub fn new() -> Self {
        Runtime::default()
    }
}

impl raw::Runtime for Runtime {
    type Stop = Stop;
    type Wait = Wait;

    type Error = Error;

    fn spawn<A: raw::Actor>(&mut self, actor: A) -> Option<raw::Spawned<A>> {
        // Create a new context for the actor.
        let mut ctx = A::Context::new();

        // Create a new `Spawned` struct from
        // the actor's context.
        let spawned = raw::Spawned::new(&mut ctx);

        // Generate the actor's ID.
        let id = self.rng.next_u64();

        // Create the actor's kill channel.
        let (sender, recver) = actor::new_kill();

        // Try to create the actor (fails if
        // it refused to start).
        let actor = Actor::new(id, actor, recver, self.sender.clone(), ctx)?;

        // Save the actor's kill channel's
        // sender.
        self.actors.insert(id, sender);

        // Spawn the actor.
        runtime::spawn(actor);

        Some(spawned)
    }

    fn stop(mut self) -> Stop {
        // Ask for each actor to stop.
        for (_, actor) in self.actors.iter_mut() {
            actor.kill();
        }

        Stop(self.wait())
    }

    fn wait(self) -> Wait {
        Wait {
            rt: self,
            errors: vec![],
        }
    }
}

impl Future for Stop {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), Error>> {
        // `Stop` is just a wrapper
        // arround `Wait` (what differs
        // is what happens before it
        // is returned by the `stop`
        // method).
        Pin::new(&mut self.get_mut().0).poll(ctx)
    }
}

impl Future for Wait {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), Error>> {
        let wait = self.get_mut();
        let rt = &mut wait.rt;

        loop {
            if rt.actors.is_empty() {
                return Poll::Ready(Ok(()));
            }

            // We try to poll from the actors'
            // kill channel's receiver.
            match Pin::new(&mut rt.recver).poll_next(ctx) {
                Poll::Ready(Some(actor)) => {
                    if rt.actors.remove(&actor).is_none() {
                        wait.errors.push(Error::already_removed(actor));
                    }
                }
                // If the channel has been closed,
                // we stop the future.
                Poll::Ready(None) => {
                    if wait.errors.len() > 1 {
                        return Poll::Ready(Err(Error::multiple(wait.errors.split_off(0))));
                    } else if wait.errors.len() == 1 {
                        return Poll::Ready(Err(wait.errors.pop().unwrap()));
                    } else {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        let (sender, recver) = actor::new_killed();

        Runtime {
            actors: FnvHashMap::default(),
            sender,
            recver,
            rng: Xoshiro512StarStar::from_entropy(),
        }
    }
}
