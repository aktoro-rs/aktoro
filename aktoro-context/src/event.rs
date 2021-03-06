use std::marker::PhantomData;

use aktoro_raw as raw;

/// A wrapper around an an event that an actor should
/// handle (this is used to allow generalization).
pub(crate) struct Event<A, E>
where
    A: raw::EventHandler<E>,
    E: Send + 'static,
{
    event: Option<E>,
    _act: PhantomData<A>,
}

impl<A, E> Event<A, E>
where
    A: raw::EventHandler<E>,
    E: Send + 'static,
{
    pub(crate) fn new(event: E) -> Self {
        Event {
            event: Some(event),
            _act: PhantomData,
        }
    }
}

impl<A, E> raw::Event for Event<A, E>
where
    A: raw::EventHandler<E>,
    E: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) -> Result<(), A::Error> {
        // If the event hasn't already been handled, we
        // do so.
        if let Some(event) = self.event.take() {
            actor.handle(event, ctx)?;
        }

        Ok(())
    }
}
