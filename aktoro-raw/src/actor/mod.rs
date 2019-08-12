use std::error;

mod handle;
mod status;

pub use self::handle::Handle;
pub use self::status::Status;

/// TODO: module documentation

/// TODO: documentation
pub trait Actor: Unpin + Send + Sized {
    /// TODO: documentation
    type Context;

    /// TODO: documentation
    type Status: Status<Self>;

    type Error: error::Error;

    /// TODO: documentation
    fn starting(&mut self, _: &mut Self::Context) -> Result<Self::Status, Self::Error> {
        Ok(Self::Status::started())
    }

    /// TODO: documentation
    fn started(&mut self, _: &mut Self::Context) -> Result<Self::Status, Self::Error> {
        Ok(Self::Status::running())
    }

    /// TODO: documentation
    fn stopping(&mut self, _: &mut Self::Context) -> Result<Self::Status, Self::Error> {
        Ok(Self::Status::stopped())
    }

    /// TODO: documentation
    fn stopped(&mut self, _: &mut Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }

    /// TODO: documentation
    fn dead(&mut self) {}
}
