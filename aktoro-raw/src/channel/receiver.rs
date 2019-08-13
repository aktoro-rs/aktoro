use std::error;

use futures_core::Stream;

/// TODO: documentation
pub trait Receiver<M, R = ()>: Stream<Item = M> + Send
where
    M: Send,
    R: Send,
{
    type Error: error::Error;

    /// TODO: documentation
    fn try_recv(&self) -> Result<Option<M>, Self::Error>;

    /// TODO: documentation
    fn close_channel(&self);

    /// TODO: documentation
    fn is_closed(&self) -> bool;

    /// TODO: documentation
    fn disconnect(&self);
}
