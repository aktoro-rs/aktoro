use futures_core::Stream;

use crate::actor::Actor;
use crate::channel::Sender;
use crate::control::Controller;
use crate::control::Update;
use crate::message::Message;

pub trait Context<A: Actor>: Unpin + Send + 'static + Stream<Item = Work<A>> {
    type Controller: Controller<A>;
    type Sender: Sender<A>;

    fn new() -> Self;

    fn controller(&self) -> Self::Controller;
    fn sender(&self) -> Self::Sender;
}

pub enum Work<A: Actor> {
    Action(Box<Update<A>>),
    Event(A::Event),
    Message(Box<Message<Actor = A>>),
}
