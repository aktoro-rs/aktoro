use crossbeam_queue::ArrayQueue;
use crossbeam_queue::SegQueue;

/// The queue used by the channels to send
/// and receive data.
pub(crate) enum Queue<T> {
    /// The bounded queue variant.
    Bounded(ArrayQueue<T>),
    /// The unbounded queue variant.
    Unbounded(SegQueue<T>),
}

impl<T> Queue<T> {
    /// Pushes a message over the queue if
    /// the inner buffer isn't full.
    pub(crate) fn push(&self, msg: T) -> Result<(), T> {
        match self {
            Queue::Bounded(queue) => queue.push(msg).map_err(|err| err.0),
            Queue::Unbounded(queue) => {
                queue.push(msg);
                Ok(())
            }
        }
    }

    /// Pops a message from the queue if
    /// one is available.
    pub(crate) fn pop(&self) -> Option<T> {
        match self {
            Queue::Bounded(queue) => {
                if let Ok(msg) = queue.pop() {
                    Some(msg)
                } else {
                    None
                }
            }
            Queue::Unbounded(queue) => {
                if let Ok(msg) = queue.pop() {
                    Some(msg)
                } else {
                    None
                }
            }
        }
    }

    /// Whether the queue contains
    /// messages to be poped.
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Queue::Bounded(queue) => queue.is_empty(),
            Queue::Unbounded(queue) => queue.is_empty(),
        }
    }
}
