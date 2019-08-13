use std::marker::PhantomData;

use aktoro_raw as raw;

use crate::context::Context;
use crate::error::Error;

/// TODO: documentation
pub struct Handle<A> 
where
    A: raw::Actor<Context = Context<A>>,
{
    // TODO: ...
    _act: PhantomData<A>,
}

impl<A> Handle<A>
where
    A: raw::Actor<Context = Context<A>>,
{
    /// TODO: documentation
    ///
    /// TODO(inner): *
    pub(crate) fn new() -> Handle<A> {
        Handle {
            // TODO: ...
            _act: PhantomData,
        }
    }
}

impl<A> raw::actor::Handle<A> for Handle<A>
where
    A: raw::Actor<Context = Context<A>>,
{
    type Error = Error;

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn stop(&self) -> Result<(), Self::Error> {
        Ok(()) // TODO
    }

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn kill(&self) -> Result<(), Self::Error> {
        Ok(()) // TODO
    }

    /// TODO: documentation
    ///
    /// TODO(inner): *
    fn send<M>(&self, msg: M) -> Result<(), Self::Error>
    where
        A: raw::handler::message::Handler<M>,
        M: raw::handler::Message,
    {
        Ok(()) // TODO
    }
}

impl<A> Clone for Handle<A>
where
    A: raw::Actor<Context = Context<A>>,
{
    fn clone(&self) -> Self {
        Handle {
            _act: PhantomData,
        }
    }
}
