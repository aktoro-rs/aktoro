use std::error;

use super::notify::Notify;
use super::response::Response;

/// TODO: documentation
pub trait Sender<M, R = ()>: Send
where
    M: Send,
    R: Send,
{
    /// TODO: documentation
    type Notify: Notify;

    /// TODO: documentation
    type Response: Response<R>;

    type Error: error::Error;

    /// TODO: documentation
    fn try_send(&self, msg: M) -> Result<(), Self::Error>;

    /// TODO: documentation
    fn try_send_resp(&self, msg: M) -> Result<Self::Response, Self::Error>;

    /// TODO: documentation
    fn try_send_notify(&self, msg: M) -> Result<Self::Notify, Self::Error>;

    /// TODO: documentation
    fn close_channel(&self);

    /// TODO: documentation
    fn is_closed(&self) -> bool;

    /// TODO: documentation
    fn disconnect(&self);
}
