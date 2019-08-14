use std::error;

mod message;
mod receiver;
mod sender;

pub mod notification;
pub mod response;

pub use self::message::Message;
pub use self::receiver::Receiver;
pub use self::sender::Sender;

/// TODO: module documentation

/// TODO: documentation
pub trait Channel<I, O= ()>: Sized {
    /// TODO: documentation
    type Config: Default;

    /// TODO: documentation
    type Message: Message<I, O>;

    /// TODO: documentation
    type Sender: Sender<Self::Message, I, O>;

    /// TODO: documentation
    type Receiver: Receiver<Self::Message, I, O>;

    type Error: error::Error;

    /// TODO: documentation
    fn new() -> Result<Self, Self::Error> {
        Self::new_with(Default::default())
    }

    /// TODO: documentation
    fn new_with(config: Self::Config) -> Result<Self, Self::Error>;

    /// TODO: documentation
    fn sender(&self) -> Result<Self::Sender, Self::Error>;

    /// TODO: documentation
    fn receiver(&self) -> Result<Self::Receiver, Self::Error>;
}
