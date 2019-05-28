use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::channel;
use aktoro_raw as raw;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use futures_util::FutureExt;

use crate::message::Message;

pub struct Sender<A: raw::Actor>(channel::Sender<Box<raw::Message<Actor = A>>>);

pub(crate) struct Receiver<A: raw::Actor>(channel::Receiver<Box<raw::Message<Actor = A>>>);

pub(crate) fn new<A: raw::Actor>() -> (Sender<A>, Receiver<A>) {
    let (sender, recver) = channel::unbounded(); // TODO: bounded OR unbounded

    (Sender(sender), Receiver(recver))
}

impl<A> raw::Sender<A> for Sender<A>
where
    A: raw::Actor,
{
    type Error = (); // FIXME

    fn send<M>(&mut self, msg: M) -> Result<BoxFuture<Result<A::Output, ()>>, ()>
    where
        A: raw::Handler<M>,
        M: Send + 'static,
    {
        let (msg, recv) = Message::new(msg);

        self.0
            .send(Box::new(msg))
            .map(|()| recv.map(|out| out.map_err(|_| ())).boxed()) // TODO: handle err
            .map_err(|_| ()) // TODO: handle err
    }
}

impl<A> raw::Receiver<A> for Receiver<A> where A: raw::Actor {}

impl<A> Stream for Receiver<A>
where
    A: raw::Actor,
{
    type Item = Box<raw::Message<Actor = A>>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}

impl<A> Clone for Sender<A>
where
    A: raw::Actor,
{
    fn clone(&self) -> Self {
        Sender(self.0.clone())
    }
}
