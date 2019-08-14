use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;

/// TODO: documentation
pub(super) struct Inner<M> {
    /// TODO: documentation
    pub(super) msg: AtomicCell<Option<M>>,
    /// TODO: documentation
    waker: AtomicCell<Option<Waker>>
}

impl<M> Inner<M> {
    /// TODO: documentation
    pub(super) fn new() -> Inner<M> {
        Inner {
            msg: AtomicCell::new(None),
            waker: AtomicCell::new(None),
        }
    }

    /// TODO: documentation
    pub(super) fn register(&self, waker: Waker) {
        self.waker.store(Some(waker));
    }

    /// TODO: documentation
    pub(super) fn unregister(&self) {
        self.waker.store(None);
    }

    /// TODO: documentation
    pub(super) fn notify(&self) {
        if let Some(waker) = self.waker.swap(None) {
            waker.wake();
        }
    }
}
