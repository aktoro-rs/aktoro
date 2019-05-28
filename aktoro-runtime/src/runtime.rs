use aktoro_raw as raw;
use aktoro_raw::Context as RawContext;

use crate::actor::Actor;

pub struct Runtime; // TODO: updates

impl raw::Runtime for Runtime {
    fn spawn<A: raw::Actor>(
        &mut self,
        actor: A,
    ) -> (
        <A::Context as raw::Context<A>>::Controller,
        <A::Context as raw::Context<A>>::Sender,
    ) {
        let ctx = A::Context::new();

        let ctrler = ctx.controller();
        let sender = ctx.sender();

        runtime::spawn(Actor::new(actor, ctx));

        (ctrler, sender)
    }
}
