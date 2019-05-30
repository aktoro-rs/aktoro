use std::marker::PhantomData;

use aktoro_raw as raw;

pub(crate) struct EventMessage<A, E>
where
    A: raw::EventHandler<E>,
    E: Send + 'static,
{
    event: Option<E>,
    _actor: PhantomData<A>,
}

impl<A, E> EventMessage<A, E>
where
    A: raw::EventHandler<E>,
    E: Send + 'static,
{
    pub(crate) fn new(event: E) -> Self {
        EventMessage {
            event: Some(event),
            _actor: PhantomData,
        }
    }
}

impl<A, E> raw::EventMessage for EventMessage<A, E>
where
    A: raw::EventHandler<E>,
    E: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) -> Result<(), A::Error> {
        if let Some(event) = self.event.take() {
            actor.handle(event, ctx)?;
        }

        Ok(())
    }
}
