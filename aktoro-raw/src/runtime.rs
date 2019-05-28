use crate::actor::Actor;
use crate::context::Context;

pub trait Runtime {
    // TODO: impl Future
    fn spawn<A: Actor>(
        &mut self,
        actor: A,
    ) -> (
        <A::Context as Context<A>>::Controller,
        <A::Context as Context<A>>::Sender,
    );
}
