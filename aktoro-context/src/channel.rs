use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel as channel;
use aktoro_channel::error::TrySendError;
use aktoro_raw as raw;
use futures_core::Stream;
use futures_util::FutureExt;

use crate::message::Message;

pub struct Sender<A: raw::Actor>(channel::Sender<Box<dyn raw::Message<Actor = A>>>);

pub struct Receiver<A: raw::Actor>(channel::Receiver<Box<dyn raw::Message<Actor = A>>>);

pub(crate) fn new<A: raw::Actor>() -> (Sender<A>, Receiver<A>) {
    let (sender, recver) = channel::Builder::new()
        .unbounded()
        .unlimited_msgs()
        .unlimited_senders()
        .unlimited_receivers()
        .build();

    (Sender(sender), Receiver(recver))
}

impl<A> raw::Sender<A> for Sender<A>
where
    A: raw::Actor,
{
    type Receiver = Receiver<A>;

    type Error = TrySendError<Box<dyn raw::Message<Actor = A>>>;

    fn try_send<M>(&mut self, msg: M) -> raw::SenderRes<A::Output, Self::Error>
    where
        A: raw::Handler<M>,
        M: Send,
    {
        let (msg, recv) = Message::new(msg);

        self.0.try_send(Box::new(msg))?;

        Ok(recv.map(Ok).boxed())
    }
}

impl<A: raw::Actor> raw::Receiver<A> for Receiver<A> {}

impl<A> Stream for Receiver<A>
where
    A: raw::Actor,
{
    type Item = Box<dyn raw::Message<Actor = A>>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}

impl<A> Clone for Sender<A>
where
    A: raw::Actor,
{
    fn clone(&self) -> Self {
        Sender(self.0.try_clone().unwrap())
    }
}
