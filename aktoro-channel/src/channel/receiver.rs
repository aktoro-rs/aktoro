use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use aktoro_raw as raw;
use aktoro_raw::channel::Receiver as RawReceiver;
use crossbeam_utils::atomic::AtomicCell;
use futures_core::FusedStream;
use futures_core::Stream;

use crate::error::Error;

use super::inner::Inner;
use super::inner::Waker;
use super::message::Message;

/// TODO: documentation
pub struct Receiver<I, O> {
	inner: Option<Arc<Inner<I, O>>>,
	waker: Waker,
}

impl<I, O> Receiver<I, O> {
    /// TODO: documentation
	pub(super) fn new(inner: Arc<Inner<I, O>>) -> Receiver<I, O> {
		let waker = AtomicCell::new((true, None));
		let waker = Arc::new(waker);

		inner.register(waker.clone());

		Receiver { inner: Some(inner), waker, }
	}
}

impl<I, O> raw::channel::Receiver<Message<I, O>, I, O> for Receiver<I, O> {
	type Error = Error;

    /// TODO: documentation
	fn try_recv(&self) -> Result<Option<Message<I, O>>, Error> {
		if let Some(inner) = &self.inner {
			inner.try_recv()
		} else {
			Err(Error::disconnected(None))
		}
	}

	/// TODO: documentation
	fn is_closed(&self) -> Result<bool, Error> {
		if let Some(inner) = &self.inner {
			Ok(inner.is_closed())
		} else {
			Err(Error::disconnected(None))
		}
	}

	/// TODO: documentation
	fn disconnect(&mut self) -> Result<(), Error> {
		self.waker.store((false, None));

		let inner = if let Some(inner) = self.inner.take() {
			inner
		} else {
			return Err(Error::disconnected(None));
		};

		if inner.counters.sub_recver() == 0 {
			inner.close();
		}

		Ok(())
	}

	/// TODO: documentation
	fn close_channel(&self) -> Result<(), Error> {
		if let Some(inner) = &self.inner {
			inner.close();
			Ok(())
		} else {
			Err(Error::disconnected(None))
		}
	}
}

impl<I, O> Stream for Receiver<I, O> {
	type Item = Message<I, O>;

	/// TODO: documentation
	fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Message<I, O>>> {
		self.waker.store((true, None));

		let inner = if let Some(inner) = &self.inner {
			inner
		} else {
			self.waker.store((false, None));
			return Poll::Ready(None);
		};

		match inner.try_recv() {
			Ok(Some(msg)) => Poll::Ready(Some(msg)),
			Ok(None) => {
				self.waker.store((true, Some(ctx.waker().clone())));
				Poll::Pending
			}
			// TODO: handle error
			Err(_) => {
				self.waker.store((false, None));
				Poll::Ready(None)
			}
		}
	}
}

impl<I, O> FusedStream for Receiver<I, O> {
	/// TODO: documentation
	fn is_terminated(&self) -> bool {
		if let Some(inner) = &self.inner {
			inner.is_closed() && inner.is_empty()
		} else {
			true
		}
	}
}

impl<I, O> Drop for Receiver<I, O> {
	fn drop(&mut self) {
		self.disconnect().ok();
	}
}
