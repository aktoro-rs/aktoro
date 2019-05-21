use crate::actor::Actor;
use crate::channel::Receiver;
use crate::control::Controlled;

pub trait Context<A: Actor> {
    type Receiver: Receiver<A>;
    type Controlled: Controlled;

    fn new() -> Self;

    fn sender(&self) -> <Self::Receiver as Receiver<A>>::Sender;
    fn controller(&self) -> <Self::Controlled as Controlled>::Controller;

    fn stop(&mut self); // TODO: ret
}
