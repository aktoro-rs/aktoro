use std::error;

/// TODO: documentation
pub trait Actor: Unpin + Send {
    /// TODO: documentation
    type Context;

    /// TODO: documentation
    type Status: Status;

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

/// TODO: documentation
pub trait Status: PartialEq + Unpin + Send {
    /// TODO: documentation
    fn starting() -> Self;

    /// TODO: documentation
    fn started() -> Self;

    /// TODO: documentation
    fn running() -> Self;

    /// TODO: documentation
    fn stopping() -> Self;

    /// TODO: documentation
    fn stopped() -> Self;

    /// TODO: documentation
    fn dead() -> Self;

    /// TODO: documentation
    fn is_starting(&self) -> bool;
    /// TODO: documentation
    fn is_running(&self) -> bool;
    /// TODO: documentation
    fn is_started(&self) -> bool;
    /// TODO: documentation
    fn is_stopping(&self) -> bool;
    /// TODO: documentation
    fn is_stopped(&self) -> bool;
    /// TODO: documentation
    fn is_dead(&self) -> bool;
}
