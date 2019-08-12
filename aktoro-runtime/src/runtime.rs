use aktoro_raw as raw;

use crate::error::Error;

/// TODO: documentation
pub struct Config;

/// TODO: documentation
pub struct Runtime;

impl raw::Runtime for Runtime {
    /// TODO: documentation
    type Config = Config;

    type Error = Error;
}

impl Default for Config {
    fn default() -> Self {
        Config
    }
}
