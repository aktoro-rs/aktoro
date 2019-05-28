use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::channel;
use aktoro_channel::once;
use aktoro_raw as raw;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use futures_util::FutureExt;

pub struct Controller<A: raw::Actor>(channel::Sender<Box<raw::Update<A>>>);

pub(crate) struct Controlled<A: raw::Actor>(channel::Receiver<Box<raw::Update<A>>>);

pub struct Update<A: raw::Actor> {
    action: A::Action,
    sender: once::Sender<A::Status>,
}

pub(crate) fn new<A: raw::Actor>() -> (Controller<A>, Controlled<A>) {
    let (sender, recver) = channel::unbounded(); // TODO: bounded OR unbounded

    (Controller(sender), Controlled(recver))
}

impl<A> raw::Controller<A> for Controller<A>
where
    A: raw::Actor,
{
    type Error = (); // FIXME

    fn send(&mut self, action: A::Action) -> Result<BoxFuture<Result<A::Status, ()>>, ()> {
        let (update, recv) = Update::new(action);

        self.0
            .send(Box::new(update))
            .map(|()| recv.map(|out| out.map_err(|_| ())).boxed()) // TODO: handle err
            .map_err(|_| ()) // TODO: handle err
    }
}

impl<A> raw::Controlled<A> for Controlled<A> where A: raw::Actor {}

impl<A> Update<A>
where
    A: raw::Actor,
{
    fn new(action: A::Action) -> (Self, once::Receiver<A::Status>) {
        let (sender, recver) = once::new();

        (Update { action, sender }, recver)
    }
}

impl<A> raw::Update<A> for Update<A>
where
    A: raw::Actor,
{
    fn action(&self) -> &A::Action {
        &self.action
    }

    fn update(&mut self, status: A::Status) {
        // TODO: Result?
        self.sender.send(status).ok().unwrap(); // FIXME: result
    }
}

impl<A> Stream for Controlled<A>
where
    A: raw::Actor,
{
    type Item = Box<raw::Update<A>>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<Box<raw::Update<A>>>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}

impl<A> Clone for Controller<A>
where
    A: raw::Actor,
{
    fn clone(&self) -> Self {
        Controller(self.0.clone())
    }
}
