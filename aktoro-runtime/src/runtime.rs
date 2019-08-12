use aktoro_raw as raw;

use crate::error::Error;

/// TODO: documentation
pub struct Config;

/// TODO: documentation
pub struct Runtime;

/// TODO: documentation
#[derive(Clone)]
pub struct Handle;

impl raw::Runtime for Runtime {
    /// TODO: documentation
    type Config = Config;

    /// TODO: documentation
    type Handle = Handle;

    type Error = Error;

    /// TODO: documentation
    ///
    /// TODO(inner): use config
    fn init_with(config: Config) -> Result<Runtime, Error> {
        Ok(Runtime)
    }

    /// TODO: documentation
    ///
    /// TODO(inner): create handle
    fn handle(&self) -> Handle {
        Handle
    }

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn wait(self) -> Result<(), Error> {
        Ok(()) // TODO
    }

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn stop(self) -> Result<(), Error> {
        Ok(()) // TODO
    }
}

impl raw::runtime::Handle for Handle {}

impl Default for Config {
    fn default() -> Self {
        Config
    }
}
