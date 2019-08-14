use std::error;

use futures_core::Stream;

use super::message::Message;

/// TODO: documentation
pub trait Receiver<M, I, O = ()>: Stream<Item = M>
where
    M: Message<I, O>,
{
    type Error: error::Error;

    /// TODO: documentation
    fn try_recv(&self) -> Result<Option<M>, Self::Error>;

    /// TODO: documentation
    fn is_closed(&self) -> Result<bool, Self::Error>;

    /// TODO: documentation
    fn disconnect(&mut self) -> Result<(), Self::Error>;

    /// TODO: documentation
    fn close_channel(&self) -> Result<(), Self::Error>;
}
