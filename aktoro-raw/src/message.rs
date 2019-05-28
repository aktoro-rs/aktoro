use crate::actor::Actor;

pub trait Message: Send {
    type Actor: Actor;

    fn handle(&mut self, actor: &mut Self::Actor, ctx: &mut <Self::Actor as Actor>::Context); // TODO: Result?
}
