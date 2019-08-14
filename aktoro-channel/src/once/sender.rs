use std::sync::Arc;

use crate::error::Error;

use super::inner::Inner;

/// TODO: documentation
pub(crate) struct Sender<M> {
    inner: Option<Arc<Inner<M>>>,
}

impl<M> Sender<M> {
    /// TODO: documentation
    pub(super) fn new(inner: Arc<Inner<M>>) -> Sender<M> {
        Sender { inner: Some(inner), }
    }

    /// TODO: documentation
    pub(crate) fn try_send(&mut self, msg: M) -> Result<(), Error<M>> {
        let inner = if let Some(inner) = self.inner.take() {
            inner
        } else {
            return Err(Error::closed(Some(msg)));
        };

        inner.msg.store(Some(msg));
        inner.notify();

        Ok(())
    }
}
