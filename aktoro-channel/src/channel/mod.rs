use std::sync::Arc;

use aktoro_raw as raw;

use crate::error::Error;

mod config;
mod counters;
mod inner;
mod message;
mod queue;
mod receiver;
mod sender;

use self::inner::Inner;

pub use self::config::Config;
pub use self::message::Message;
pub use self::receiver::Receiver;
pub use self::sender::Sender;

/// TODO: module documentation

/// TODO: documentation
pub struct Channel<I, O = ()> {
	/// TODO: documentation
	inner: Arc<Inner<I, O>>,
}

impl<I, O> raw::Channel<I, O> for Channel<I, O> {
	/// TODO: documentation
	type Config = Config;

	/// TODO: documentation
	type Message = Message<I, O>;

	/// TODO: documentation
	type Sender = Sender<I, O>;

	/// TODO: documentation
	type Receiver = Receiver<I, O>;

	type Error = Error;

	/// TODO: documentation
	///
	/// TODO(inner): use configuration
	fn new_with(config: Config) -> Result<Channel<I, O>, Error> {
		let inner = Inner::new(config);
		let inner = Arc::new(inner);

		Ok(Channel { inner, })
	}

	/// TODO: documentation
	fn sender(&self) -> Result<Sender<I, O>, Error> {
		if self.inner.counters.add_sender().is_ok() {
			Ok(Sender::new(self.inner.clone()))
		} else {
			Err(Error::sender_limit())
		}
	}

	/// TODO: documentation
	fn receiver(&self) -> Result<Receiver<I, O>, Error> {
		if self.inner.counters.add_recver().is_ok() {
			Ok(Receiver::new(self.inner.clone()))
		} else {
			Err(Error::recver_limit())
		}
	}
}
