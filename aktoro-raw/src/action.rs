use crate::actor::Actor;

pub trait ActionMessage: Send {
    type Actor: Actor;

    fn handle(&mut self, actor: &mut Self::Actor, ctx: &mut <Self::Actor as Actor>::Context)
        -> Result<(), <Self::Actor as Actor>::Error>;
}

pub trait ActionHandler<A: Send + 'static>: Actor {
    type Output: Send;

    fn handle(&mut self, action: A, ctx: &mut Self::Context) -> Result<Self::Output, Self::Error>;
}
