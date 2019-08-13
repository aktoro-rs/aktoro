use std::error;
use std::future::Future;

/// TODO: documentation
pub trait Response<R>: Future<Output = R>
where
    R: Send,
{
    type Error: error::Error;

    /// TODO: documentation
    fn send(&self, response: R) -> Result<(), Self::Error>;
}
