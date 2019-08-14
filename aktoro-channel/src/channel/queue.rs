use crossbeam_queue::ArrayQueue;
use crossbeam_queue::SegQueue;

use crate::error::Error;

/// TODO: documentation
pub(crate) struct Queue<T> {
    inner: QueueInner<T>,
}

/// TODO: documentation
enum QueueInner<T> {
    Bounded(ArrayQueue<T>),
    Unbounded(SegQueue<T>),
}

impl<T> Queue<T> {
    /// TODO: documentation
    pub(crate) fn bounded(cap: usize) -> Queue<T> {
        Queue {
            inner: QueueInner::Bounded(ArrayQueue::new(cap)),
        }
    }

    /// TODO: documentation
    pub(crate) fn unbounded() -> Queue<T> {
        Queue {
            inner: QueueInner::Unbounded(SegQueue::new()),
        }
    }

    /// TODO: documentation
    pub(crate) fn is_empty(&self) -> bool {
        match &self.inner {
            QueueInner::Bounded(queue) => queue.is_empty(),
            QueueInner::Unbounded(queue) => queue.is_empty(),
        }
    }

    /// TODO: documentation
    pub(crate) fn push(&self, value: T) -> Result<(), Error<T>> {
        match &self.inner {
            QueueInner::Bounded(queue) => queue.push(value).map_err(|err| Error::full(err.0)),
            QueueInner::Unbounded(queue) => {
                queue.push(value);
                Ok(())
            }
        }
    }

    /// TODO: documentation
    pub(crate) fn pop(&self) -> Option<T> {
        match &self.inner {
            QueueInner::Bounded(queue) => {
                if let Ok(value) = queue.pop() {
                    Some(value)
                } else {
                    None
                }
            }
            QueueInner::Unbounded(queue) => {
                if let Ok(value) = queue.pop() {
                    Some(value)
                } else {
                    None
                }
            }
        }
    }
}
