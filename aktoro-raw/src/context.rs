use futures_core::Stream;

use crate::action::ActionMessage;
use crate::actor::Actor;
use crate::channel::Sender;
use crate::control::Controller;
use crate::event::EventHandler;
use crate::event::EventMessage;
use crate::message::Message;
use crate::update::Updater;

pub trait Context<A: Actor>: Unpin + Send + 'static + Stream<Item = Work<A>> {
    type Controller: Controller<A>;
    type Sender: Sender<A>;
    type Updater: Updater<A>;

    fn new() -> Self;

    fn kill(&mut self);
    fn killed(&self) -> bool;

    fn emit<E>(&mut self, event: E)
    where
        A: EventHandler<E>,
        E: Send + 'static;

    fn status(&self) -> &A::Status;
    fn update(&mut self, status: A::Status);

    fn controller(&self) -> &Self::Controller;
    fn sender(&self) -> &Self::Sender;
    fn updated(&mut self) -> Option<<Self::Updater as Updater<A>>::Updated>;

    fn controlled(&mut self) -> &mut <Self::Controller as Controller<A>>::Controlled;
    fn receiver(&mut self) -> &mut <Self::Sender as Sender<A>>::Receiver;
    fn updater(&mut self) -> &mut Self::Updater;
}

pub enum Work<A: Actor> {
    Action(Box<ActionMessage<Actor = A>>),
    Event(Box<EventMessage<Actor = A>>),
    Message(Box<Message<Actor = A>>),
}
