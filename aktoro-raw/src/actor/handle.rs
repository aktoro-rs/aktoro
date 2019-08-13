use std::error;

use crate::actor::Actor;
use crate::handler::message;
use crate::handler::Message;

/// TODO: documentation
pub trait Handle<A: Actor>: Unpin + Clone + Send {
    type Error: error::Error;

    /// TODO: documentation
    ///
    /// TODO(return): future
    fn stop(&self) -> Result<(), Self::Error>;

    /// TODO: documentation
    ///
    /// TODO(return): future
    fn kill(&self) -> Result<(), Self::Error>;


    /// TODO: documentation
    ///
    /// TODO(return): future
    fn send<M>(&self, msg: M) -> Result<(), Self::Error>
    where
        A: message::Handler<M>,
        M: Message + 'static;
}
