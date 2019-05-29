use std::future::Future;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_raw as raw;
use aktoro_raw::Context as RawContext;
use fnv::FnvHashMap;
use futures_core::Stream;
use rand_core::RngCore;
use rand_core::SeedableRng;
use rand_xoshiro::Xoshiro512StarStar;

use crate::actor;
use crate::actor::Kill;
use crate::actor::Killed;
use crate::actor::Killing;

pub struct Runtime {
    actors: FnvHashMap<u64, Kill>,
    killing: Killing,
    killed: Killed,
    rng: Xoshiro512StarStar,
}

pub struct Stop(Wait);

pub struct Wait(Runtime);

impl Runtime {
    pub fn init() -> Runtime {
        let (killing, killed) = actor::new_kill();

        Runtime {
            actors: FnvHashMap::default(),
            killing,
            killed,
            rng: Xoshiro512StarStar::seed_from_u64(42), // FIXME
        }
    }
}

impl raw::Runtime for Runtime {
    type Stop = Stop;
    type Wait = Wait;

    type Error = (); // FIXME

    fn spawn<A: raw::Actor>(
        &mut self,
        actor: A,
    ) -> (
        <A::Context as raw::Context<A>>::Controller,
        <A::Context as raw::Context<A>>::Sender,
        <<A::Context as raw::Context<A>>::Updater as raw::Updater<A>>::Updated,
    ) {
        let mut ctx = A::Context::new();

        let ctrler = ctx.controller().clone();
        let sender = ctx.sender().clone();
        let updted = ctx.updated().unwrap(); // FIXME

        let id = self.rng.next_u64();

        let (actor, kill) = actor::new(id, actor, self.killing.clone(), ctx);

        self.actors.insert(id, kill);

        runtime::spawn(actor);

        (ctrler, sender, updted)
    }

    fn stop(mut self) -> Stop {
        for (_, actor) in self.actors.iter_mut() {
            actor.kill();
        }

        Stop(self.wait())
    }

    fn wait(self) -> Wait {
        Wait(self)
    }
}

impl Future for Stop {
    type Output = Result<(), ()>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), ()>> {
        Pin::new(&mut self.get_mut().0).poll(ctx)
    }
}

impl Future for Wait {
    type Output = Result<(), ()>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), ()>> {
        let rt = &mut self.get_mut().0;

        loop {
            if rt.actors.len() == 0 {
                return Poll::Ready(Ok(()));
            }

            match Pin::new(&mut rt.killed).poll_next(ctx) {
                Poll::Ready(Some(actor)) => {
                    rt.actors.remove(&actor).unwrap(); // FIXME
                }
                Poll::Ready(None) => unimplemented!(), // FIXME
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
