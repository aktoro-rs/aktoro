use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task;

use crossbeam_queue::SegQueue;
use crossbeam_utils::atomic::AtomicCell;

use crate::counters::Counters;
use crate::error::*;
use crate::message::Message;
use crate::queue::Queue;

type Waker = Arc<AtomicCell<(bool, Option<task::Waker>)>>;

/// A channel allowing senders to pass
/// messages over it, and receivers to
/// retrieve them.
pub(crate) struct Channel<T> {
    /// The queue that is holding the
    /// messages that have not been
    /// received yet.
    pub(crate) queue: Queue<Message<T>>,
    /// Whether the channel is closed.
    pub(crate) closed: AtomicBool,
    /// The counters used to store the
    /// current number of senders,
    /// receivers, messages and the
    /// channel's limits.
    pub(crate) counters: Counters,
    /// A list of the wakers that can
    /// be used to wake up receivers.
    pub(crate) wakers: SegQueue<Waker>,
}

impl<T> Channel<T> {
    /// Tries to send a message over the
    /// channel.
    pub(crate) fn try_send(&self, msg: Message<T>) -> Result<(), TrySendError<T>> {
        // If the channel has already
        // been closed, we return an
        // error.
        if self.is_closed() {
            return Err(TrySendError::closed(msg.msg));
        }

        // If we couldn't increase the
        // number of messages inside the
        // inner counters, we return an
        // error.
        if self.counters.add_msg().is_err() {
            return Err(TrySendError::limit(msg.msg));
        }

        // We try to push the message
        // over the queue.
        self.queue
            .push(msg)
            .map_err(|msg| TrySendError::full(msg.msg))?;

        // We notify a receiver that a
        // new message is available.
        self.notify();

        Ok(())
    }

    /// Tries to receive a message from the
    /// channel if one is available.
    pub(crate) fn try_recv(&self) -> Result<Option<Message<T>>, TryRecvError> {
        // If the queue is empty, we
        // return an error if it's closed.
        if self.queue.is_empty() {
            if self.check_is_closed() {
                Err(TryRecvError::closed())
            } else {
                Ok(None)
            }
        // Otherwise, we pop try to
        // pop a message from it (it
        // could return `None` if the
        // message was already poped).
        } else {
            Ok(self.queue.pop())
        }
    }

    /// Registers a new waker.
    pub(crate) fn register(&self, waker: Waker) {
        self.wakers.push(waker);
    }

    /// Notifies a waker if one is
    /// available.
    fn notify(&self) {
        if let Ok(waker) = self.wakers.pop() {
            match waker.swap((true, None)) {
                (true, Some(waker_)) => {
                    self.wakers.push(waker);
                    waker_.wake();
                }
                (true, None) => {
                    self.wakers.push(waker);
                    self.notify();
                }
                _ => self.notify(),
            }
        }
    }

    /// Whether the queue is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Whether the channel has been
    /// closed.
    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Verifies whether the channel has
    /// been closed and checks if senders
    /// are still connected to it (closing
    /// the channel if not).
    pub(crate) fn check_is_closed(&self) -> bool {
        if self.is_closed() {
            return true;
        }

        if self.counters.senders() == 0 {
            self.close();
            true
        } else {
            false
        }
    }

    /// Closes the channel.
    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}
