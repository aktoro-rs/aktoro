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

pub struct Runtime {
    actors: FnvHashMap<u64, Kill>,
    sender: KilledSender,
    recver: KilledRecver,
    rng: Xoshiro512StarStar,
}

pub struct Stop(Wait);

pub struct Wait {
    rt: Runtime,
    errors: Vec<Error>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime::default()
    }
}

impl raw::Runtime for Runtime {
    type Stop = Stop;
    type Wait = Wait;

    type Error = Error;

    fn spawn<A: raw::Actor>(&mut self, actor: A) -> Option<raw::Spawned<A>> {
        let mut ctx = A::Context::new();

        let spawned = raw::Spawned::new(&mut ctx);

        let id = self.rng.next_u64();

        let (sender, recver) = actor::new_kill();

        let actor = Actor::new(id, actor, recver, self.sender.clone(), ctx)?;

        self.actors.insert(id, sender);

        runtime::spawn(actor);

        Some(spawned)
    }

    fn stop(mut self) -> Stop {
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

            match Pin::new(&mut rt.recver).poll_next(ctx) {
                Poll::Ready(Some(actor)) => {
                    if rt.actors.remove(&actor).is_none() {
                        wait.errors.push(Error::already_removed(actor));
                    }
                }
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
