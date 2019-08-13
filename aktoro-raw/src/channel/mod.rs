use std::error;

mod notify;
mod receiver;
mod response;
mod sender;

pub use self::receiver::Receiver;
pub use self::sender::Sender;

/// TODO: module documentation

/// TODO: documentation
pub trait Channel<M, R = ()>: Sized
where
    M: Send,
    R: Send,
{
    /// TODO: documentation
    type Config: Default;

    /// TODO: documentation
    type Sender: Sender<M, R>;

    /// TODO: documentation
    type Receiver: Receiver<M, R>;

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
