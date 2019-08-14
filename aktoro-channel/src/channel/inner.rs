use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task;

use crossbeam_utils::atomic::AtomicCell;
use crossbeam_queue::SegQueue;

use crate::error::Error;

use super::config::Config;
use super::counters::Counters;
use super::message::Message;
use super::queue::Queue;

pub(crate) type Waker = Arc<AtomicCell<(bool, Option<task::Waker>)>>;

/// TODO: documentation
pub(crate) struct Inner<I, O> {
    /// TODO: documentation
    queue: Queue<Message<I, O>>,
    /// TODO: documentation
    closed: AtomicBool,
    /// TODO: documentation
    pub(crate) counters: Counters,
    /// TODO: documentation
    wakers: SegQueue<Waker>,
}

impl<I, O> Inner<I, O> {
    /// TODO: documentation
    ///
    /// TODO(inner): use config
    pub(super) fn new(config: Config) -> Inner<I, O> {
        let queue = Queue::unbounded(); // TODO: use config
        let closed = AtomicBool::new(false);
        let counters = Counters::new(None, None, None); // TODO: use config
        let wakers = SegQueue::new();

        Inner { queue, closed, counters, wakers, }
    }

    /// TODO: documentation
    pub(crate) fn try_send(&self, msg: Message<I, O>) -> Result<(), Error<Message<I, O>>> {
        if self.is_closed() {
            return Err(Error::closed(Some(msg)));
        }

        if self.counters.add_msg().is_err() {
            return Err(Error::msg_limit(msg));
        }

        self.queue.push(msg)?;

        self.notify();

        Ok(())
    }

    /// TODO: documentation
    pub(crate) fn try_recv(&self) -> Result<Option<Message<I, O>>, Error> {
        if self.queue.is_empty() {
            if self.check_is_closed() {
                Err(Error::closed(None))
            } else {
                Ok(None)
            }
        } else {
            Ok(self.queue.pop())
        }
    }

    /// TODO: documentation
    pub(crate) fn register(&self, waker: Waker) {
        self.wakers.push(waker);
    }

    /// TODO: documentation
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

    /// TODO: documentation
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// TODO: documentation
    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// TODO: documentation
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

    /// TODO: documentation
    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}
