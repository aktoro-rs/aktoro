use crossbeam_queue::ArrayQueue;
use crossbeam_queue::SegQueue;

pub(crate) enum Queue<T> {
    Bounded(ArrayQueue<T>),
    Unbounded(SegQueue<T>),
}

impl<T> Queue<T> {
    pub(crate) fn push(&self, msg: T) -> Result<(), T> {
        match self {
            Queue::Bounded(queue) => queue.push(msg).map_err(|err| err.0),
            Queue::Unbounded(queue) => {
                queue.push(msg);
                Ok(())
            }
        }
    }

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

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Queue::Bounded(queue) => queue.is_empty(),
            Queue::Unbounded(queue) => queue.is_empty(),
        }
    }
}
