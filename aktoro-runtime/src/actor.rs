use std::future::Future;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel as channel;
use aktoro_channel::error::TrySendError;
use aktoro_channel::Notify;
use aktoro_channel::Receiver;
use aktoro_channel::Sender;
use aktoro_raw as raw;
use aktoro_raw::Context as RawContext;
use aktoro_raw::Status as RawStatus;
use futures_core::Stream;

use crate::error::Error;

pub(crate) struct Actor<A: raw::Actor> {
    id: u64,
    act: A,
    ctx: A::Context,
    started: bool,
    kill: KillRecver,
    killed: KilledSender,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Status {
    Starting,
    Started,
    Stopping,
    Stopped,
    Dead,
}

pub(crate) struct KillSender(Option<Notify>);
pub(crate) struct KillRecver(Option<Notify>);

pub(crate) struct KilledSender(Sender<u64>);
pub(crate) struct KilledRecver(Receiver<u64>);

pub(crate) fn new_kill() -> (KillSender, KillRecver) {
    let notify = Notify::new();

    (KillSender(Some(notify.0)), KillRecver(Some(notify.1)))
}

pub(crate) fn new_killed() -> (KilledSender, KilledRecver) {
    let (sender, recver) = channel::Builder::new()
        .unbounded()
        .unlimited_msgs()
        .unlimited_senders()
        .unlimited_receivers()
        .build();

    (KilledSender(sender), KilledRecver(recver))
}

impl<A: raw::Actor> Actor<A> {
    pub(crate) fn new(
        id: u64,
        mut act: A,
        kill: KillRecver,
        killed: KilledSender,
        mut ctx: A::Context,
    ) -> Option<Self> {
        ctx.set_status(A::Status::starting());
        act.starting(&mut ctx);

        if ctx.status().is_stopping() {
            act.stopping(&mut ctx);

            if ctx.status().is_stopped() {
                ctx.set_status(A::Status::stopped());
            }
        }

        if ctx.status().is_stopped() {
            act.stopped(&mut ctx);
            ctx.set_status(A::Status::dead());
        }

        if ctx.status().is_dead() {
            return None;
        }

        Some(Actor {
            id,
            act,
            ctx,
            started: false,
            kill,
            killed,
        })
    }

    fn dead(&mut self) -> Result<(), Error> {
        self.ctx.set_status(A::Status::dead());

        if let Err(err) = self.ctx.update() {
            return Err(Box::new(err).into());
        }

        if let Err(err) = self.killed.killed(self.id) {
            return Err(Box::new(err).into());
        }

        Ok(())
    }
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

    fn dead() -> Status {
        Status::Dead
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

    fn is_dead(&self) -> bool {
        self == &Status::Dead
    }
}

impl KillSender {
    pub(crate) fn kill(&mut self) {
        if let Some(notify) = self.0.take() {
            notify.done();
            self.0 = None;
        }
    }
}

impl KilledSender {
    fn killed(&mut self, id: u64) -> Result<(), TrySendError<u64>> {
        self.0.try_send(id)
    }
}

impl<A: raw::Actor> Future for Actor<A> {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Self::Output> {
        let actor = self.get_mut();

        loop {
            match Pin::new(&mut actor.kill).poll(ctx) {
                Poll::Ready(()) => actor.act.stopped(&mut actor.ctx),
                Poll::Pending => (),
            }

            if actor.ctx.status().is_stopping() {
                actor.act.stopping(&mut actor.ctx);

                if actor.ctx.status().is_stopping() {
                    actor.ctx.set_status(A::Status::stopped());
                }
            }

            if actor.ctx.status().is_stopped() {
                actor.act.stopped(&mut actor.ctx);
                return Poll::Ready(actor.dead());
            }

            if actor.ctx.status().is_dead() {
                return Poll::Ready(actor.dead());
            }

            if !actor.started {
                actor.ctx.set_status(A::Status::started());
                actor.act.started(&mut actor.ctx);
                actor.started = true;
                continue;
            }

            match Pin::new(&mut actor.ctx).poll_next(ctx) {
                Poll::Ready(Some(work)) => match work {
                    raw::Work::Action(mut action) => {
                        if let Err(err) = action.handle(&mut actor.act, &mut actor.ctx) {
                            return Poll::Ready(Err(Error::std(err).add_res(actor.dead())));
                        }

                        continue;
                    }
                    raw::Work::Event(mut event) => {
                        if let Err(err) = event.handle(&mut actor.act, &mut actor.ctx) {
                            return Poll::Ready(Err(Error::std(err).add_res(actor.dead())));
                        }

                        continue;
                    }
                    raw::Work::Message(mut msg) => {
                        if let Err(err) = msg.handle(&mut actor.act, &mut actor.ctx) {
                            return Poll::Ready(Err(Error::std(err).add_res(actor.dead())));
                        }

                        continue;
                    }
                    raw::Work::Update => continue,
                },
                Poll::Ready(None) => {
                    actor.ctx.set_status(A::Status::stopped());
                    continue;
                }
                Poll::Pending => (),
            }

            return Poll::Pending;
        }
    }
}

impl Future for KillRecver {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<()> {
        let recver = self.get_mut();

        if let Some(notify) = &mut recver.0 {
            if Pin::new(notify).poll(ctx).is_ready() {
                recver.0.take();
            } else {
                return Poll::Pending;
            }
        }

        Poll::Ready(())
    }
}

impl Stream for KilledRecver {
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

impl Clone for KilledSender {
    fn clone(&self) -> Self {
        KilledSender(self.0.try_clone().unwrap())
    }
}
