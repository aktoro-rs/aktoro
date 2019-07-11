use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel as channel;
use aktoro_channel::error::TrySendError;
use aktoro_raw as raw;
use futures_core::Stream;
use futures_util::FutureExt;

use crate::action::Action;

/// An actor's control channel sender, used
/// by [`Context`].
///
/// [`Context`]: struct.Context.html
pub struct Controller<A: raw::Actor>(channel::Sender<Box<dyn raw::Action<Actor = A>>>);

/// An actor's control channel receiver,
/// used by [`Context`].
///
/// [`Context`]: struct.Context.html
pub struct Controlled<A: raw::Actor>(channel::Receiver<Box<dyn raw::Action<Actor = A>>>);

/// Creates a new control channel for the
/// specified actor type, returning a sender
/// and receiver connected to it.
pub(crate) fn new<A: raw::Actor>() -> (Controller<A>, Controlled<A>) {
    // TODO: maybe allow the channel's configuration
    // to be specified.
    let (sender, recver) = channel::Builder::new()
        .unbounded()
        .unlimited_msgs()
        .unlimited_senders()
        .unlimited_receivers()
        .build();

    (Controller(sender), Controlled(recver))
}

impl<A> raw::Controller<A> for Controller<A>
where
    A: raw::Actor + 'static,
{
    type Controlled = Controlled<A>;

    type Error = TrySendError<Box<dyn raw::Action<Actor = A>>>;

    fn try_send<D>(&mut self, action: D) -> raw::ControllerRes<A::Output, Self::Error>
    where
        A: raw::ActionHandler<D>,
        D: Send + 'static,
    {
        let (action, recv) = Action::new(action);

        self.0.try_send(Box::new(action))?;

        Ok(recv.map(Ok).boxed())
    }
}

impl<A: raw::Actor> raw::Controlled<A> for Controlled<A> {}

impl<A> Stream for Controlled<A>
where
    A: raw::Actor,
{
    type Item = Box<dyn raw::Action<Actor = A>>;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<Option<Box<dyn raw::Action<Actor = A>>>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}

impl<A> Clone for Controller<A>
where
    A: raw::Actor,
{
    fn clone(&self) -> Self {
        Controller(self.0.try_clone().unwrap())
    }
}
