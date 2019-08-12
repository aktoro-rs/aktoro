use aktoro_raw as raw;
use aktoro_raw::actor::Status as RawStatus;

use crate::error::Error;

/// TODO: documentation
pub struct Config;

/// TODO: documentation
pub struct Context<A>
where
    A: raw::Actor,
{
    /// TODO: documentation
    actor_id: u64,
    /// TODO: documentation
    status: A::Status,
}

impl<A> raw::Context<A> for Context<A>
where
    A: raw::Actor,
{
    /// TODO: documentation
    type Config = Config;

    type Error = Error;

    /// TODO: documentation
    ///
    /// TODO(inner): use config
    fn new(actor_id: u64, config: Config) -> Result<Self, Error> {
        Ok(Context {
            actor_id,
            status: A::Status::starting(),
        })
    }

    /// TODO: documentation
    fn status(&self) -> &A::Status {
        &self.status
    }
}

impl Default for Config {
    fn default() -> Self {
        Config
    }
}
