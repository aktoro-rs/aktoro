use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task;
use std::task::Poll;
use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;

/// A sender or receiver, with the sender
/// being able to store an output once and
/// to notify the receiver, and the receiver
/// being able to register a `Waker` to be
/// waked up after the output is available.
pub(crate) struct Respond<O>(Arc<RespondInner<O>>);

/// The inner "channel" of `Respond`.
struct RespondInner<O> {
    /// The output that the sender has sent
    /// or `None` if no output has been
    /// sent yet, or if the receiver has
    /// already read it.
    out: AtomicCell<Option<O>>,
    /// A pointer to an optional waker that
    /// will be notified when the output
    /// is available.
    waker: AtomicCell<Option<Waker>>,
}

impl<O> Respond<O> {
    /// Creates a new channel, returning a
    /// sender and a receiver.
    pub(crate) fn new() -> (Self, Self) {
        let inner = Arc::new(RespondInner {
            out: AtomicCell::new(None),
            waker: AtomicCell::new(None),
        });

        (Respond(inner.clone()), Respond(inner))
    }

    /// Stores the output and eventually
    /// notifies the receiver.
    pub(crate) fn respond(self, out: O) {
        self.0.out.store(Some(out));

        if let Some(waker) = self.0.waker.swap(None) {
            waker.wake();
        }
    }
}

impl<O> Future for Respond<O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<O> {
        // If the output is available, we
        // complete the future with it.
        if let Some(out) = self.0.out.swap(None) {
            Poll::Ready(out)
        // Otherwise, we store the future's
        // waker.
        } else {
            self.0.waker.store(Some(ctx.waker().clone()));
            Poll::Pending
        }
    }
}
