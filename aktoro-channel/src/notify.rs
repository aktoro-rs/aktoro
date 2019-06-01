use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;

pub struct Notify(Arc<Inner>);

struct Inner {
    done: AtomicBool,
    waker: AtomicCell<Option<Waker>>,
}

impl Notify {
    pub(crate) fn new() -> Self {
        Notify(Arc::new(Inner {
            done: AtomicBool::new(false),
            waker: AtomicCell::new(None),
        }))
    }

    pub fn is_done(&self) -> bool {
        self.0.done.load(Ordering::SeqCst)
    }

    pub(crate) fn register(&self, waker: Waker) {
        self.0.waker.store(Some(waker));
    }

    pub(crate) fn done(self) {
        self.0.done.store(true, Ordering::SeqCst);

        if let Some(waker) = self.0.waker.swap(None) {
            waker.wake();
        }
    }

    pub(crate) fn clone(&self) -> Self {
        Notify(self.0.clone())
    }
}

impl Future for Notify {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        if self.is_done() {
            Poll::Ready(())
        } else {
            self.register(ctx.waker().clone());
            Poll::Pending
        }
    }
}
