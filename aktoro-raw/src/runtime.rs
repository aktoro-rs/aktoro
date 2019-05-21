use crate::actor::Actor;
use crate::channel::Receiver;
use crate::context::Context;
use crate::control::Controlled;

pub trait Runtime {
    fn spawn<A: Actor, C: Context<A>>(&mut self, act: A)
        -> (<C::Receiver as Receiver<A>>::Sender, <C::Controlled as Controlled>::Controller);
}
