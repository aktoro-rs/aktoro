use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use futures_core::Stream;

use crate::action::ActionHandler;
use crate::actor::Actor;
use crate::channel::Sender as RawSender;
use crate::channel::SenderRes;
use crate::context::Context;
use crate::control::Controller as RawController;
use crate::control::ControllerRes;
use crate::message::Handler;
use crate::update::Updater;

type Sender<A> = <<A as Actor>::Context as Context<A>>::Sender;
type SenderError<A> = <Sender<A> as RawSender<A>>::Error;

type Controller<A> = <<A as Actor>::Context as Context<A>>::Controller;
type ControllerError<A> = <Controller<A> as RawController<A>>::Error;

type Updated<A> = <<<A as Actor>::Context as Context<A>>::Updater as Updater<A>>::Updated;

/// A wrapper around an actor's
/// message, control and update
/// channels.
pub struct Spawned<A: Actor> {
    sender: Sender<A>,
    ctrler: Controller<A>,
    updted: Option<Updated<A>>,
}

impl<A: Actor> Spawned<A> {
    /// Creates a new `Spawned` struct from an actor's
    /// context.
    pub fn new(ctx: &mut A::Context) -> Spawned<A> {
        Spawned {
            sender: ctx.sender().clone(),
            ctrler: ctx.controller().clone(),
            updted: Some(
                ctx.updated()
                    .expect("ctx.updated() can't return None this early"),
            ),
        }
    }

    /// Tries to send a message over the actor's
    /// message channel, returning a future
    /// resolving with the result returned by the
    /// message handler.
    pub fn try_send_msg<M>(&mut self, msg: M) -> SenderRes<A::Output, SenderError<A>>
    where
        A: Handler<M>,
        M: Send,
    {
        self.sender.try_send(msg)
    }

    /// Tries send an action over the actor's
    /// control channel, returning a future resolving
    /// with the result returned by the action 
    /// handler.
    pub fn try_send_action<D>(&mut self, action: D) -> ControllerRes<A::Output, ControllerError<A>>
    where
        A: ActionHandler<D>,
        D: Send + 'static,
    {
        self.ctrler.try_send(action)
    }

    /// Returns a reference to the actor's message
    /// channel sender.
    pub fn sender(&self) -> &Sender<A> {
        &self.sender
    }

    /// Returns a reference to the actor's control
    /// channel sender.
    pub fn controller(&self) -> &Controller<A> {
        &self.ctrler
    }

    /// Tries to return a mutable reference to the
    /// actor's update channel receiver.
    pub fn updated_ref(&mut self) -> Option<&mut Updated<A>> {
        self.updted.as_mut()
    }

    /// Tries to return the actor's update channel
    /// receiver.
    pub fn updated(&mut self) -> Option<Updated<A>> {
        self.updted.take()
    }
}

impl<A: Actor> Unpin for Spawned<A> {}

impl<A> Stream for Spawned<A>
where
    A: Actor,
    Updated<A>: Unpin,
{
    type Item = A::Status;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<A::Status>> {
        if let Some(updted) = &mut self.get_mut().updted {
            Pin::new(updted).poll_next(ctx)
        } else {
            Poll::Ready(None)
        }
    }
}

impl<A> Clone for Spawned<A>
where
    A: Actor,
{
    fn clone(&self) -> Self {
        Spawned {
            sender: self.sender.clone(),
            ctrler: self.ctrler.clone(),
            updted: None,
        }
    }
}
