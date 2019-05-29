use crate::actor::Actor;

pub trait ActionMessage: Send {
    type Actor: Actor;

    fn handle(&mut self, actor: &mut Self::Actor, ctx: &mut <Self::Actor as Actor>::Context); // TODO: Result?
}

pub trait ActionHandler<A: Send + 'static>: Actor {
    fn handle(&mut self, action: A, ctx: &mut Self::Context);
}
