use std::future::Future;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::channel;
use aktoro_channel::once;
use aktoro_raw as raw;
use aktoro_raw::Context as AkContext;
use aktoro_raw::Status as AkStatus;
use futures_core::Stream;

pub(crate) struct Actor<A: raw::Actor> {
    id: u64,
    act: A,
    ctx: A::Context,
    started: bool,
    kill: once::Receiver<()>,
    killing: Killing,
}

#[derive(Eq, PartialEq, Clone)]
pub enum Status {
    Starting,
    Started,
    Stopping,
    Stopped,
}

pub(crate) struct Kill(once::Sender<()>);

#[derive(Clone)]
pub(crate) struct Killing(channel::Sender<u64>); // TODO: err?

pub(crate) struct Killed(channel::Receiver<u64>); // TODO: err?

pub(crate) fn new<A: raw::Actor>(
    id: u64,
    mut act: A,
    mut killing: Killing,
    mut ctx: A::Context,
) -> Option<(Actor<A>, Kill)> {
    ctx.update(A::Status::starting());
    act.starting(&mut ctx);

    if ctx.status().is_stopping() {
        act.stopping(&mut ctx);
    }

    if ctx.status().is_stopping() {
        ctx.update(A::Status::stopped());
        act.stopped(&mut ctx);
        killing.0.send(id).unwrap(); // FIXME
        return None;
    } else if ctx.status().is_stopped() {
        killing.0.send(id).unwrap(); // FIXME
        return None;
    }

    let (_kill, kill) = once::new();

    Some((
        Actor {
            id,
            act,
            ctx,
            started: false,
            kill,
            killing,
        },
        Kill(_kill),
    ))
}

pub(crate) fn new_kill() -> (Killing, Killed) {
    let (killing, killed) = channel::unbounded(); // TODO: bounded OR unbounded

    (Killing(killing), Killed(killed))
}

impl raw::Status for Status {
    fn starting() -> Status {
        Status::Starting
    }

    fn started() -> Status {
        Status::Started
    }

    fn stopping() -> Status {
        Status::Stopping
    }

    fn stopped() -> Status {
        Status::Stopped
    }

    fn is_starting(&self) -> bool {
        self == &Status::Starting
    }

    fn is_started(&self) -> bool {
        self == &Status::Started
    }

    fn is_stopping(&self) -> bool {
        self == &Status::Stopping
    }

    fn is_stopped(&self) -> bool {
        self == &Status::Stopped
    }
}

impl<A: raw::Actor> Future for Actor<A> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<()> {
        let actor = self.get_mut();

        loop {
            match Pin::new(&mut actor.kill).poll(ctx) {
                Poll::Ready(Ok(())) => {
                    actor.act.stopped(&mut actor.ctx);
                    actor.killing.0.send(actor.id).unwrap(); // FIXME
                    return Poll::Ready(());
                }
                Poll::Ready(Err(_)) => { // TODO: handle err
                    actor.act.stopped(&mut actor.ctx);
                    actor.killing.0.send(actor.id).unwrap(); // FIXME
                    return Poll::Ready(());
                }
                Poll::Pending => (),
            }

            if actor.ctx.status().is_stopping() {
                actor.act.stopping(&mut actor.ctx);
            }

            if actor.ctx.status().is_stopping() {
                actor.ctx.update(A::Status::stopped());
                actor.act.stopped(&mut actor.ctx);
                actor.killing.0.send(actor.id).unwrap(); // FIXME
                return Poll::Ready(());
            } else if actor.ctx.status().is_stopped() {
                actor.killing.0.send(actor.id).unwrap(); // FIXME
                return Poll::Ready(());
            }

            if !actor.started {
                actor.ctx.update(A::Status::started());
                actor.act.started(&mut actor.ctx);
                actor.started = true;
                continue;
            }

            match Pin::new(&mut actor.ctx).poll_next(ctx) {
                Poll::Ready(Some(work)) => match work {
                    raw::Work::Action(mut action) => {
                        action.handle(&mut actor.act, &mut actor.ctx).ok().unwrap(); // FIXME
                        continue;
                    }
                    raw::Work::Event(mut event) => {
                        event.handle(&mut actor.act, &mut actor.ctx).ok().unwrap(); // FIXME
                        continue;
                    }
                    raw::Work::Message(mut msg) => {
                        msg.handle(&mut actor.act, &mut actor.ctx).ok().unwrap(); // FIXME
                        continue;
                    }
                    raw::Work::Update => continue,
                },
                Poll::Ready(None) => {
                    actor.ctx.update(A::Status::stopped());
                    actor.act.stopped(&mut actor.ctx);
                    continue;
                }
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

impl Default for Status {
    fn default() -> Status {
        Status::Starting
    }
}
