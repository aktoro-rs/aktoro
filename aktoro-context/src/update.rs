use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel as channel;
use aktoro_channel::error::TrySendError;
use aktoro_raw as raw;
use futures_core::Stream;

pub struct Updater<A: raw::Actor>(channel::Sender<A::Status>);

pub struct Updated<A: raw::Actor>(channel::Receiver<A::Status>);

pub(crate) fn new<A: raw::Actor>() -> (Updater<A>, Updated<A>) {
    let (sender, recver) = channel::Builder::new()
        .unbounded()
        .unlimited_msgs()
        .unlimited_senders()
        .unlimited_receivers()
        .build();

    (Updater(sender), Updated(recver))
}

impl<A> raw::Updater<A> for Updater<A>
where
    A: raw::Actor,
{
    type Updated = Updated<A>;

    type Error = TrySendError<A::Status>;

    fn try_send(&mut self, status: A::Status) -> Result<(), Self::Error> {
        self.0.try_send(status)
    }
}

impl<A: raw::Actor> raw::Updated<A> for Updated<A> {}

impl<A> Stream for Updated<A>
where
    A: raw::Actor,
{
    type Item = A::Status;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<A::Status>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}
