use std::pin::Pin;
use std::task;
use std::task::Poll;

use aktoro_channel as channel;
use aktoro_channel::error::TrySendError;
use aktoro_raw as raw;
use futures_core::Stream;

/// A wrapper around an actor's status,
/// containing its identifier.
pub struct Update<A>
where
    A: raw::Actor,
{
    /// The actor's identifier.
    actor_id: u64,
    /// The actor's new status.
    status: A::Status,
}

/// An actor's update channel sender, used by
/// [`Context`].
///
/// [`Context`]: struct.Context.html
pub struct Updater<A: raw::Actor>(channel::Sender<Update<A>>);

/// An actor's update channel receiver, used
/// by [`Context`].
///
/// [`Context`]: struct.Context.html
pub struct Updated<A: raw::Actor>(channel::Receiver<Update<A>>);

/// Creates a new control channel for the
/// specified actor type, returning a sender
/// and receiver connected to it.
pub(crate) fn new<A: raw::Actor>() -> (Updater<A>, Updated<A>) {
    // TODO: maybe allow the channel's configuration
    // to be specified.
    let (sender, recver) = channel::Builder::new()
        .unbounded()
        .unlimited_msgs()
        .unlimited_senders()
        .unlimited_receivers()
        .build();

    (Updater(sender), Updated(recver))
}

impl<A> Update<A>
where
    A: raw::Actor,
{
    pub(crate) fn new(actor_id: u64, status: A::Status) -> Self {
        Update { actor_id, status }
    }
}

impl<A> raw::Status for Update<A>
where
    A: raw::Actor,
{
    fn starting() -> Self {
        Update {
            actor_id: !0,
            status: A::Status::starting(),
        }
    }

    fn started() -> Self {
        Update {
            actor_id: !0,
            status: A::Status::started(),
        }
    }

    fn stopping() -> Self {
        Update {
            actor_id: !0,
            status: A::Status::stopping(),
        }
    }

    fn stopped() -> Self {
        Update {
            actor_id: !0,
            status: A::Status::stopped(),
        }
    }

    fn dead() -> Self {
        Update {
            actor_id: !0,
            status: A::Status::dead(),
        }
    }

    fn is_starting(&self) -> bool {
        self.status.is_starting()
    }

    fn is_started(&self) -> bool {
        self.status.is_started()
    }

    fn is_stopping(&self) -> bool {
        self.status.is_stopping()
    }

    fn is_stopped(&self) -> bool {
        self.status.is_stopped()
    }

    fn is_dead(&self) -> bool {
        self.status.is_dead()
    }
}

impl<A> raw::Update for Update<A>
where
    A: raw::Actor,
{
    fn actor_id(&self) -> u64 {
        self.actor_id
    }

    fn set_actor_id(&mut self, id: u64) {
        self.actor_id = id;
    }
}

impl<A> raw::Updater<A> for Updater<A>
where
    A: raw::Actor + 'static,
{
    type Update = Update<A>;

    type Updated = Updated<A>;

    type Error = TrySendError<Update<A>>;

    fn try_send(&mut self, update: Update<A>) -> Result<(), Self::Error> {
        self.0.try_send(update)
    }
}

impl<A: raw::Actor> raw::Updated<Update<A>> for Updated<A> {}

impl<A> Stream for Updated<A>
where
    A: raw::Actor,
{
    type Item = Update<A>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Option<Update<A>>> {
        Pin::new(&mut self.get_mut().0).poll_next(ctx)
    }
}

impl<A> PartialEq for Update<A>
where
    A: raw::Actor,
{
    fn eq(&self, other: &Self) -> bool {
        self.actor_id == other.actor_id && self.status == other.status
    }
}

impl<A> Default for Update<A>
where
    A: raw::Actor,
{
    fn default() -> Self {
        Update {
            actor_id: !0,
            status: A::Status::default(),
        }
    }
}

impl<A> Clone for Update<A>
where
    A: raw::Actor,
{
    fn clone(&self) -> Self {
        Update {
            actor_id: self.actor_id,
            status: self.status.clone(),
        }
    }
}
