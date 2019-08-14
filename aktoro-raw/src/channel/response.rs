use std::error;
use std::future::Future;

/// TODO: documentation
pub trait Respond<R> {
    /// TODO: documentation
	type Response: Response<R>;

	type Error: error::Error;

    /// TODO: documentation
	fn send(self, response: R) -> Result<(), Self::Error>;
}

/// TODO: documentation
pub trait Response<R>: Future<Output = Option<R>> {}
