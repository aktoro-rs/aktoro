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
use runtime::task::JoinHandle;

use crate::actor;
use crate::actor::Actor;
use crate::actor::KillSender as Kill;
use crate::actor::KilledRecver;
use crate::actor::KilledSender;
use crate::error::Error;
use crate::net::NetworkManager;

/// An actor runtime using the [`runtime`] crate.
///
/// [`runtime`]: https://docs.rs/runtime
pub struct Runtime {
    /// A map matching an actor's ID with
    /// a sender for its kill channel and
    /// a handle for it.
    actors: FnvHashMap<u64, (Kill, JoinHandle<Result<(), Error>>)>,
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
    type NetworkManager = NetworkManager;

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

        // Spawn the actor.
        let handle = runtime::spawn(actor);

        // Save the actor's kill channel's
        // sender.
        self.actors.insert(id, (sender, handle));

        Some(spawned)
    }

    fn net(&mut self) -> NetworkManager {
        NetworkManager
    }

    /// Asks to all the actors managed by the
    /// runtime to stop, returning a future
    /// resolving after all of them have been
    /// stopped.
    ///
    /// ## Note
    ///
    /// Calling this method and polling the
    /// returned future might be required to
    /// poll the actors a first time, making
    /// this method kind of useless if that's
    /// the case.
    fn stop(mut self) -> Stop {
        // Ask for each actor to stop.
        for (_, actor) in self.actors.iter_mut() {
            actor.0.kill();
        }

        Stop(self.wait())
    }

    /// Waits for all the actors to be stopped,
    /// returning a future waiting for it.
    ///
    /// ## Note
    ///
    /// Calling this method and polling the
    /// returned future might be required to
    /// poll the actors a first time.
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

            // We poll all the actors' handle.
            let mut remove = vec![];
            for (id, act) in rt.actors.iter_mut() {
                if let Poll::Ready(res) = Pin::new(&mut act.1).poll(ctx) {
                    remove.push(*id);

                    if let Err(err) = res {
                        wait.errors.push(err);
                    }
                }
            }

            // We remove the dead actors.
            for actor in &remove {
                if rt.actors.remove(actor).is_none() {
                    wait.errors.push(Error::already_removed(*actor));
                }
            }

            // We restart the loop if actors
            // were removed in case other
            // actors stopped in the mean
            // time.
            if remove.len() > 0 {
                continue;
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
