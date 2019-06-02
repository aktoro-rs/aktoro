use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::task::Waker;

use crossbeam_queue::SegQueue;

use crate::counters::Counters;
use crate::error::*;
use crate::message::Message;
use crate::queue::Queue;

pub(crate) struct Channel<T> {
    pub(crate) queue: Queue<Message<T>>,
    pub(crate) closed: AtomicBool,
    pub(crate) counters: Counters,
    pub(crate) wakers: SegQueue<Waker>,
}

impl<T> Channel<T> {
    pub(crate) fn try_send(&self, msg: Message<T>) -> Result<(), TrySendError<T>> {
        if self.is_closed() {
            return Err(TrySendError::closed(msg.msg));
        }

        if self.counters.add_msg().is_err() {
            return Err(TrySendError::limit(msg.msg));
        }

        self.queue
            .push(msg)
            .map_err(|msg| TrySendError::full(msg.msg))?;

        self.notify();

        Ok(())
    }

    pub(crate) fn try_recv(&self) -> Result<Option<Message<T>>, TryRecvError> {
        if self.queue.is_empty() {
            if self.check_is_closed() {
                Err(TryRecvError::closed())
            } else {
                Ok(None)
            }
        } else {
            Ok(self.queue.pop())
        }
    }

    pub(crate) fn register(&self, waker: Waker) {
        self.wakers.push(waker);
    }

    fn notify(&self) {
        if let Ok(waker) = self.wakers.pop() {
            waker.wake();
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

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

    pub(crate) fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}
