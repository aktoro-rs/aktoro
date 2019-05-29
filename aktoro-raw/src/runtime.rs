use std::future::Future;

use crate::actor::Actor;
use crate::context::Context;
use crate::update::Updater;

pub trait Runtime {
    type Stop: Future<Output = Result<(), Self::Error>>;
    type Wait: Future<Output = Result<(), Self::Error>>;

    type Error;

    fn spawn<A: Actor>(
        &mut self,
        actor: A,
    ) -> (
        <A::Context as Context<A>>::Controller,
        <A::Context as Context<A>>::Sender,
        <<A::Context as Context<A>>::Updater as Updater<A>>::Updated,
    );

    fn stop(self) -> Self::Stop;

    fn wait(self) -> Self::Wait;
}
