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

// TODO
pub struct Wait(Runtime);

impl Runtime {
    /// Creates a new `Runtime`.
    pub fn new() -> Self {
        Runtime::default()
    }
}

impl raw::Runtime for Runtime {
    type NetworkManager = NetworkManager;

    type Wait = Wait;

    type Error = Error;

    fn actors(&self) -> Vec<u64> {
        self.actors.keys().copied().collect()
    }

    fn spawn<A>(&mut self, actor: A) -> Option<raw::Spawned<A>>
    where
        A: raw::Actor + 'static,
    {
        // Generate the actor's ID.
        let id = self.rng.next_u64();

        // Create a new context for the actor.
        let mut ctx = A::Context::new(id);

        // Create a new `Spawned` struct from
        // the actor's context.
        let spawned = raw::Spawned::new(&mut ctx);

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

    // TODO
    fn wait(self) -> Wait {
        Wait(self)
    }

    // TODO
    fn stop(&mut self) {
        // Ask to every actor to stop.
        for (_, actor) in self.actors.iter_mut() {
            actor.0.kill();
        }
    }
}

impl raw::Wait<Runtime> for Wait {
    fn runtime(&self) -> &Runtime {
        &self.0
    }

    fn into_runtime(self) -> Runtime {
        self.0
    }
}

impl Stream for Wait {
    type Item = Result<u64, (u64, Error)>;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<Option<Result<u64, (u64, Error)>>> {
        let rt = &mut self.get_mut().0;

        if rt.actors.is_empty() {
            return Poll::Ready(None);
        }

        // TODO
        let mut remove = None;
        for (id, act) in rt.actors.iter_mut() {
            if let Poll::Ready(res) = Pin::new(&mut act.1).poll(ctx) {
                remove = Some((*id, res));
            }
        }

        // TODO
        if let Some((id, res)) = remove {
            let removed = rt.actors.remove(&id);

            match (removed, res) {
                (Some(_), Err(err)) => return Poll::Ready(Some(Err((id, err)))),
                (None, Err(err)) => {
                    return Poll::Ready(Some(Err((
                        id,
                        Error::multiple(vec![Error::already_removed(id), Error::std(err)]),
                    ))))
                }
                _ => return Poll::Ready(Some(Ok(id))),
            }
        }

        // TODO
        match Pin::new(&mut rt.recver).poll_next(ctx) {
            Poll::Ready(Some(actor)) => {
                if rt.actors.remove(&actor).is_none() {
                    return Poll::Ready(Some(Err((actor, Error::already_removed(actor)))));
                } else {
                    return Poll::Ready(Some(Ok(actor)));
                }
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => (),
        }

        Poll::Pending
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
