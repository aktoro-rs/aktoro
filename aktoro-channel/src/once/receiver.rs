use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use super::inner::Inner;

/// TODO: documentation
pub(crate) struct Receiver<M> {
    inner: Option<Arc<Inner<M>>>,
}

impl<M> Receiver<M> {
    /// TODO: documentation
    pub(super) fn new(inner: Arc<Inner<M>>) -> Receiver<M> {
        Receiver { inner: Some(inner), }
    }
}

impl<M> Future for Receiver<M> {
    type Output = Option<M>;

    /// TODO: documentation
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<M>> {
        let recver = self.get_mut();
        let inner = if let Some(inner) = recver.inner.take() {
            inner
        } else {
            return Poll::Ready(None);
        };

        inner.unregister();
        if let Some(msg) = inner.msg.swap(None) {
            return Poll::Ready(Some(msg));
        }

        inner.register(ctx.waker().clone());
        recver.inner = Some(inner);

        Poll::Pending
    }
}
