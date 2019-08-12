use std::error;

use futures_core::Stream;

/// TODO: documentation
///
/// TODO(method): spawn actor
/// TODO(method): spawn actor with context config
/// TODO(trait, method): network
/// TODO(impl): stream
/// TODO(method): stop
pub trait Runtime {
    /// TODO: documentation
    ///
    /// TODO(trait): eventually a `RuntimeConfig` trait
    type Config: Default;

    type Error: error::Error;
}
