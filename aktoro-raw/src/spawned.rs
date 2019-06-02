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

pub struct Spawned<A: Actor> {
    sender: Sender<A>,
    ctrler: Controller<A>,
    updted: Option<Updated<A>>,
}

impl<A: Actor> Spawned<A> {
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

    pub fn try_send_msg<M>(&mut self, msg: M) -> SenderRes<A::Output, SenderError<A>>
    where
        A: Handler<M>,
        M: Send,
    {
        self.sender.try_send(msg)
    }

    pub fn try_send_action<D>(&mut self, action: D) -> ControllerRes<A::Output, ControllerError<A>>
    where
        A: ActionHandler<D>,
        D: Send + 'static,
    {
        self.ctrler.try_send(action)
    }

    pub fn sender(&self) -> &Sender<A> {
        &self.sender
    }

    pub fn controller(&self) -> &Controller<A> {
        &self.ctrler
    }

    pub fn updated(&mut self) -> Option<Updated<A>> {
        self.updted.take()
    }

    pub fn updated_ref(&mut self) -> Option<&mut Updated<A>> {
        self.updted.as_mut()
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
