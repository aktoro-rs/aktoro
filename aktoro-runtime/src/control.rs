use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::channel;
use aktoro_raw as raw;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use futures_util::FutureExt;

use crate::action;

pub struct Controller<A: raw::Actor>(channel::Sender<Box<raw::ActionMessage<Actor = A>>>);

pub struct Controlled<A: raw::Actor>(channel::Receiver<Box<raw::ActionMessage<Actor = A>>>);

pub(crate) fn new<A: raw::Actor>() -> (Controller<A>, Controlled<A>) {
    let (sender, recver) = channel::unbounded(); // TODO: bounded OR unbounded

    (Controller(sender), Controlled(recver))
}

impl<A> raw::Controller<A> for Controller<A>
where
    A: raw::Actor,
{
    type Controlled = Controlled<A>;

    type Error = (); // FIXME

    fn send<D>(&mut self, action: D) -> Result<BoxFuture<Result<(), ()>>, ()>
    where
        A: raw::ActionHandler<D>,
        D: Send + 'static,
    {
        let (action, recv) = action::new(action);

        self.0.send(Box::new(action)).map_err(|_| ())?; // TODO: handle err

        Ok(recv.map(|out| out.map_err(|_| ())).boxed()) // TODO: handle err
    }
}

impl<A> raw::Controlled<A> for Controlled<A> where A: raw::Actor {}

impl<A> Stream for Controlled<A>
where
    A: raw::Actor,
{
    type Item = Box<raw::ActionMessage<Actor = A>>;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<Option<Box<raw::ActionMessage<Actor = A>>>> {
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
