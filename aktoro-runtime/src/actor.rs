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

/// A wrapper around an actor and its
/// context.
pub(crate) struct Actor<A: raw::Actor> {
    id: u64,
    act: A,
    ctx: A::Context,
    started: bool,
    kill: KillRecver,
    killed: KilledSender,
}

#[derive(Eq, PartialEq, Debug, Clone)]
/// A default implementation for the
/// [`aktoro-raw::Status`] trait.
///
/// [`aktoro-raw::Status`]: https://docs.rs/aktoro-raw/struct.Status.html
pub enum Status {
    /// The status that an actor should have
    /// before [`Actor::starting`] is called.
    ///
    /// [`Actor::starting`]: https://docs.rs/aktoro-raw/trait.Actor.html#method.starting
    Starting,
    /// The status that an actor should have
    /// before [`Actor::started`] is called.
    ///
    /// [`Actor::started`]: https://docs.rs/aktoro-raw/trait.Actor.html#method.started
    Started,
    /// The status that an actor should have
    /// before [`Actor::stopping`] is called.
    ///
    /// [`Actor::stopped`]: https://docs.rs/aktoro-raw/trait.Actor.html#method.stopping
    Stopping,
    /// The status that an actor should have
    /// before [`Actor::stopped`] is called.
    ///
    /// [`Actor::stopped`]: https://docs.rs/aktoro-raw/trait.Actor.html#method.stopped
    Stopped,
    /// The status that an actor has after
    /// [`Actor::stopped`] has been called.
    ///
    /// [`Actor::stopped`]: https://docs.rs/aktoro-raw/trait.Actor.html#method.stropped
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
        // Sets the actor's status as starting
        // and call the `starting` method on it.
        ctx.set_status(A::Status::starting());
        act.starting(&mut ctx);

        // If the actor has decided to stop
        // gracefully, we call its
        // `stopping` method.
        if ctx.status().is_stopping() {
            act.stopping(&mut ctx);

            // If it has stopped, we set
            // its status as stopped.
            if ctx.status().is_stopping() {
                ctx.set_status(A::Status::stopped());
            }
        }

        // If the actor's status is marked
        // as stopped, we call its `stopped`
        // method and change its status
        // as dead.
        if ctx.status().is_stopped() {
            act.stopped(&mut ctx);
            ctx.set_status(A::Status::dead());
        }

        // If the actor is dead, we don't
        // spawn it in the background.
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

    /// Marks the actor as dead.
    fn dead(&mut self) -> Result<(), Error> {
        // We set the actor's status as
        // dead.
        self.ctx.set_status(A::Status::dead());

        // We try to notify the actor's
        // death over the killed channel.
        if let Err(err) = self.killed.killed(self.id) {
            return Err(Box::new(err).into());
        }

        // We try to push the actor's
        // new status over its update
        // channel.
        // NOTE: this is done after sending
        //   the death notification because
        //   if the `Spanwed` linked to
        //   this actor has been dropped,
        //   it will return an error.
        if let Err(err) = self.ctx.update() {
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
    /// Kills the actor by sending a
    /// message over its kill channel.
    pub(crate) fn kill(&mut self) {
        if let Some(notify) = self.0.take() {
            notify.done();
            self.0 = None;
        }
    }
}

impl KilledSender {
    /// Notifies that the actor died.
    fn killed(&mut self, id: u64) -> Result<(), TrySendError<u64>> {
        self.0.try_send(id)
    }
}

impl<A: raw::Actor> Future for Actor<A> {
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Self::Output> {
        let actor = self.get_mut();

        loop {
            // If the actor has been asked
            // to die, we kill it.
            match Pin::new(&mut actor.kill).poll(ctx) {
                Poll::Ready(()) => actor.act.stopped(&mut actor.ctx),
                Poll::Pending => (),
            }

            // If the actor's status is marked
            // as stopping, we call `stopping`
            // on it.
            if actor.ctx.status().is_stopping() {
                actor.act.stopping(&mut actor.ctx);

                // If the actor's status hasn't
                // changed, we set it as stopped.
                if actor.ctx.status().is_stopping() {
                    actor.ctx.set_status(A::Status::stopped());
                }
            }

            // If the actor's status is marked
            // as stopped, we call `stopped`
            // ont it and notify that the actor
            // is dead over the killed channel.
            if actor.ctx.status().is_stopped() {
                actor.act.stopped(&mut actor.ctx);
                return Poll::Ready(actor.dead());
            }

            // If the actor's status is marked
            // as dead, we notify that the actor
            // is dead over the killed channel.
            if actor.ctx.status().is_dead() {
                return Poll::Ready(actor.dead());
            }

            // If `started` hasn't been called
            // on the actor, we call it.
            if !actor.started {
                actor.ctx.set_status(A::Status::started());
                actor.act.started(&mut actor.ctx);
                // We save that the actor's
                // `started` method has been
                // called.
                actor.started = true;
                continue;
            }

            match Pin::new(&mut actor.ctx).poll_next(ctx) {
                Poll::Ready(Some(work)) => match work {
                    // If the context received an
                    // action for the actor to handle,
                    // we do so.
                    raw::Work::Action(mut action) => {
                        if let Err(err) = action.handle(&mut actor.act, &mut actor.ctx) {
                            return Poll::Ready(Err(Error::std(err).add_res(actor.dead())));
                        }

                        continue;
                    }
                    // If the context has been asked
                    // to get an event handled by the
                    // actor, we do so.
                    raw::Work::Event(mut event) => {
                        if let Err(err) = event.handle(&mut actor.act, &mut actor.ctx) {
                            return Poll::Ready(Err(Error::std(err).add_res(actor.dead())));
                        }

                        continue;
                    }
                    // If the context has received a
                    // message for the actor to handle,
                    // we do so.
                    raw::Work::Message(mut msg) => {
                        if let Err(err) = msg.handle(&mut actor.act, &mut actor.ctx) {
                            return Poll::Ready(Err(Error::std(err).add_res(actor.dead())));
                        }

                        continue;
                    }
                    raw::Work::Update => continue,
                },
                // If the actor's context `Work`
                // stream has been closed, we
                // change the actor's status as
                // being stopped.
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
