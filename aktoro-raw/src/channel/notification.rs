use std::error;
use std::future::Future;

/// TODO: documentation
pub trait Notify {
    /// TODO: documentation
	type Received: Received;

	type Error: error::Error;

    /// TODO: documentation
	fn notify(self) -> Result<(), Self::Error>;
}

/// TODO: documentation
pub trait Received: Future<Output = bool> {}
