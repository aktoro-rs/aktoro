use std::error;

use crate::actor::Actor;

/// TODO: module documentation

/// TODO: documentation
///
/// TODO(method): spawn actor
/// TODO(method): spawn actor with context config
/// TODO(trait, method): network
pub trait Runtime: Sized {
    /// TODO: documentation
    ///
    /// TODO(trait): eventually a `RuntimeConfig` trait
    type Config: Default;

    type Handle: Handle;

    type Error: error::Error;

    /// TODO: documentation
    fn init() -> Result<Self, Self::Error> {
        Self::init_with(Default::default())
    }

    /// TODO: documentation
    fn init_with(config: Self::Config) -> Result<Self, Self::Error>;

    /// TODO: documentation
    fn handle(&self) -> Self::Handle;

    /// TODO: documentation
    fn wait(self) -> Result<(), Self::Error>;

    /// TODO: documentation
    fn stop(self) -> Result<(), Self::Error>;
}

/// TODO: documentation
pub trait Handle: Unpin + Clone + Send {
    type Error: error::Error;

    /// TODO: documentation
    fn spawn<A: Actor>(&self) -> Result<(), Self::Error>;
}
