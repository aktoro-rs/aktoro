use std::error::Error as StdError;
use std::future::Future;

use crate::actor::Actor;
use crate::spawned::Spawned;

pub trait Runtime {
    type Stop: Future<Output = Result<(), Self::Error>>;
    type Wait: Future<Output = Result<(), Self::Error>>;

    type Error: StdError;

    fn spawn<A: Actor>(&mut self, actor: A) -> Option<Spawned<A>>;

    fn stop(self) -> Self::Stop;

    fn wait(self) -> Self::Wait;
}
