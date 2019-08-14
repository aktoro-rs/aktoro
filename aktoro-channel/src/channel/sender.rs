use std::sync::Arc;

use aktoro_raw as raw;
use aktoro_raw::channel::Message as RawMessage;
use aktoro_raw::channel::Sender as RawSender;

use crate::error::Error;
use crate::notification::Received;
use crate::response::Response;

use super::inner::Inner;
use super::message::Message;

/// TODO: documentation
pub struct Sender<I, O> {
	inner: Option<Arc<Inner<I, O>>>,
}

impl<I, O> Sender<I, O> {
    /// TODO: documentation
	pub(super) fn new(inner: Arc<Inner<I, O>>) -> Sender<I, O> {
		Sender { inner: Some(inner), }
	}
}

impl<I, O> raw::channel::Sender<Message<I, O>, I, O> for Sender<I, O> {
	type Error = Error<I>;

	/// TODO: documentation
	fn try_send(&self, msg: I) -> Result<(), Error<I>> {
		let inner = if let Some(inner) = &self.inner {
			inner
		} else {
			return Err(Error::disconnected(Some(msg)));
		};

		let msg = Message::normal(msg);
		inner.try_send(msg)
			.map_err(|err| err.map(|msg| msg.into_msg()))?;

		Ok(())
	}

	/// TODO: documentation
	fn try_send_notifying(&self, msg: I) -> Result<Received, Error<I>> {
		let inner = if let Some(inner) = &self.inner {
			inner
		} else {
			return Err(Error::disconnected(Some(msg)));
		};

		let (msg, received) = Message::notifying(msg);
		inner.try_send(msg)
			.map_err(|err| err.map(|msg| msg.into_msg()))?;

		Ok(received)
	}

	/// TODO: documentation
	fn try_send_responding(&self, msg: I) -> Result<Response<O>, Error<I>> {
		let inner = if let Some(inner) = &self.inner {
			inner
		} else {
			return Err(Error::disconnected(Some(msg)));
		};

		let (msg, response) = Message::responding(msg);
		inner.try_send(msg)
			.map_err(|err| err.map(|msg| msg.into_msg()))?;

		Ok(response)
	}

	/// TODO: documentation
	fn is_disconnected(&self) -> bool {
		self.inner.is_some()
	}

	/// TODO: documentation
	fn is_closed(&self) -> Result<bool, Error<I>> {
		if let Some(inner) = &self.inner {
			Ok(inner.is_closed())
		} else {
			Err(Error::disconnected(None))
		}
	}

	/// TODO: documentation
	fn disconnect(&mut self) -> Result<(), Error<I>> {
		let inner = if let Some(inner) = self.inner.take() {
			inner
		} else {
			return Err(Error::disconnected(None));
		};

		if inner.counters.sub_sender() == 0 {
			inner.close();
		}

		Ok(())
	}

	/// TODO: documentation
	fn close_channel(&self) -> Result<(), Error<I>> {
		if let Some(inner) = &self.inner {
			inner.close();
			Ok(())
		} else {
			Err(Error::disconnected(None))
		}
	}
}

impl<I, O> Drop for Sender<I, O> {
	fn drop(&mut self) {
		self.disconnect().ok();
	}
}
