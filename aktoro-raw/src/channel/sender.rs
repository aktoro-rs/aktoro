use std::error;

use super::message::Message;
use super::notification::Notify;
use super::response::Respond;

/// TODO: documentation
pub trait Sender<M, I, O = ()>
where
	M: Message<I, O>,
{
    type Error: error::Error;

    /// TODO: documentation
    fn try_send(&self, msg: I) -> Result<(), Self::Error>;

    /// TODO: documentation
    fn try_send_notifying(&self, msg: I) -> Result<<M::Notify as Notify>::Received, Self::Error>;

    /// TODO: documentation
    fn try_send_responding(&self, msg: I) -> Result<<M::Respond as Respond<O>>::Response, Self::Error>;

    /// TODO: documentation
    fn is_disconnected(&self) -> bool;

    /// TODO: documentation
    fn is_closed(&self) -> Result<bool, Self::Error>;

	/// TODO: documentation
	fn disconnect(&mut self) -> Result<(), Self::Error>;

    /// TODO: documentation
    fn close_channel(&self) -> Result<(), Self::Error>;
}
