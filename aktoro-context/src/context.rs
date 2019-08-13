use aktoro_raw as raw;
use aktoro_raw::actor::Status as RawStatus;

use crate::error::Error;
use crate::handle::Handle;

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
    A: raw::Actor<Context = Self>
{
    /// TODO: documentation
    type Config = Config;

    type Error = Error;

    /// TODO: documentation
    type Handle = Handle<A>;

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


    /// TODO: documentation
    ///
    /// TODO(return): an handle to the result
    /// TODO(inner): *
    fn exec<D>(&self, action: D) -> Result<(), Error>
    where
        A: raw::handler::action::Handler<D>,
        D: raw::handler::Action,
    {
        Ok(()) // TODO
    }

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn emit<E>(&self, event: E) -> Result<(), Error>
    where
        A: raw::handler::event::Handler<E>,
        E: raw::handler::Event,
    {
        Ok(()) // TODO
    }

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn handle(&self) -> Result<Handle<A>, Error> {
        Ok(Handle::new()) // TODO
    }

    /// TODO: documentation
    ///
    /// TODO(param): link type
    /// TODO(inner): *
    fn link<H, C>(&self, handle: &H) -> Result<(), Error>
    where
        H: raw::actor::Handle<C>,
        C: raw::Actor,
    {
        Ok(()) // TODO
    }
}

impl Default for Config {
    fn default() -> Self {
        Config
    }
}
