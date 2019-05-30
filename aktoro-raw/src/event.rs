use crate::actor::Actor;

pub trait EventMessage: Send {
    type Actor: Actor;

    fn handle(&mut self, actor: &mut Self::Actor, ctx: &mut <Self::Actor as Actor>::Context)
        -> Result<(), <Self::Actor as Actor>::Error>;
}

pub trait EventHandler<E: Send + 'static>: Actor {
    fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<(), Self::Error>;
}
