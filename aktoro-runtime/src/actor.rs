use std::future::Future;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::channel;
use aktoro_channel::once;
use aktoro_raw as raw;
use aktoro_raw::Context;
use futures_core::Stream;

pub(crate) struct Actor<A: raw::Actor> {
    id: u64,
    act: A,
    ctx: A::Context,
    kill: once::Receiver<()>,
    killing: Killing,
}

pub(crate) struct Kill(once::Sender<()>);

#[derive(Clone)]
pub(crate) struct Killing(channel::Sender<u64>);

pub(crate) struct Killed(channel::Receiver<u64>);

pub(crate) fn new<A: raw::Actor>(
    id: u64,
    actor: A,
    killing: Killing,
    ctx: A::Context,
) -> (Actor<A>, Kill) {
    let (_kill, kill) = once::new();

    (
        Actor {
            id,
            act: actor,
            ctx,
            kill,
            killing,
        },
        Kill(_kill),
    )
}

pub(crate) fn new_kill() -> (Killing, Killed) {
    let (killing, killed) = channel::unbounded(); // TODO: bounded OR unbounded

    (Killing(killing), Killed(killed))
}

impl<A: raw::Actor> Future for Actor<A> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<()> {
        let actor = self.get_mut();

        loop {
            if actor.ctx.killed() {
                actor.killing.0.send(actor.id).unwrap(); // FIXME

                return Poll::Ready(());
            }

            match Pin::new(&mut actor.kill).poll(ctx) {
                Poll::Ready(Ok(())) => {
                    actor.killing.0.send(actor.id).unwrap(); // FIXME

                    return Poll::Ready(());
                }
                Poll::Ready(Err(_)) => unimplemented!(), // FIXME
                Poll::Pending => (),
            }

            match Pin::new(&mut actor.ctx).poll_next(ctx) {
                Poll::Ready(Some(work)) => match work {
                    raw::Work::Action(mut action) => action.handle(&mut actor.act, &mut actor.ctx),
                    raw::Work::Event(mut event) => event.handle(&mut actor.act, &mut actor.ctx),
                    raw::Work::Message(mut msg) => msg.handle(&mut actor.act, &mut actor.ctx),
                },
                Poll::Ready(None) => unimplemented!(), // FIXME
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl Kill {
    pub(crate) fn kill(&mut self) {
        self.0.send(()).unwrap(); // FIXME
    }
}

impl Stream for Killed {
    type Item = u64;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<u64>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}
