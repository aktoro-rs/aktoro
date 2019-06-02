use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context as FutContext;
use std::task::Poll;
use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;

pub(crate) struct Respond<O>(Option<Arc<RespondInner<O>>>);

struct RespondInner<O> {
    out: AtomicCell<Option<O>>,
    waker: AtomicCell<Option<Waker>>,
}

impl<O> Respond<O> {
    pub(crate) fn new() -> (Self, Self) {
        let inner = Arc::new(RespondInner {
            out: AtomicCell::new(None),
            waker: AtomicCell::new(None),
        });

        (Respond(Some(inner.clone())), Respond(Some(inner)))
    }

    pub(crate) fn respond(&mut self, out: O) {
        if let Some(inner) = &self.0 {
            inner.out.store(Some(out));

            if let Some(waker) = inner.waker.swap(None) {
                waker.wake();
            }
        }
    }
}

impl<O> Future for Respond<O> {
    type Output = O;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<O> {
        let inner = if let Some(inner) = &mut self.get_mut().0 {
            inner
        } else {
            unreachable!();
        };

        if let Some(out) = inner.out.swap(None) {
            Poll::Ready(out)
        } else {
            inner.waker.store(Some(ctx.waker().clone()));
            Poll::Pending
        }
    }
}
