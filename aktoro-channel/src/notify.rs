use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;

/// A sender or receiver, with the sender
/// being able to notify the receiver
/// when an action has been completed and
/// the receiver being able to register
/// a `Waker` to be waked up after the
/// completion.
pub struct Notify(Arc<Inner>);

/// The inner "channel" of `Notify`.
struct Inner {
    /// Whether the sender has already
    /// completed the action and notified
    /// the receiver.
    done: AtomicBool,
    /// A pointer to an optional waker
    /// that will be notified when the
    /// action is completed.
    waker: AtomicCell<Option<Waker>>,
}

impl Notify {
    /// Creates a new channel, returning
    /// a sender and a receiver.
    pub fn new() -> (Self, Self) {
        let inner = Arc::new(Inner {
            done: AtomicBool::new(false),
            waker: AtomicCell::new(None),
        });

        (Notify(inner.clone()), Notify(inner))
    }

    /// Stores the action as bein completed,
    /// eventually waking up the receiver.
    pub fn done(self) {
        // We store the action as being completed.
        self.0.done.store(true, Ordering::SeqCst);

        // We eventually wake the receiver up.
        if let Some(waker) = self.0.waker.swap(None) {
            waker.wake();
        }
    }
}

impl Future for Notify {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        // If the action has been completed, we
        // complete the future too.
        if self.0.done.load(Ordering::SeqCst) {
            Poll::Ready(())
        // Otherwise, we register the future's
        // waker.
        } else {
            self.0.waker.store(Some(ctx.waker().clone()));
            Poll::Pending
        }
    }
}
