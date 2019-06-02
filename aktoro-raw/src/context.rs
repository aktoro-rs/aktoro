use futures_core::Stream;

use crate::action::Action;
use crate::actor::Actor;
use crate::channel::Sender;
use crate::control::Controller;
use crate::event::Event;
use crate::event::EventHandler;
use crate::message::Message;
use crate::update::Updater;

pub trait Context<A: Actor>: Unpin + Send + 'static + Stream<Item = Work<A>> {
    type Controller: Controller<A>;
    type Sender: Sender<A>;
    type Updater: Updater<A>;

    fn new() -> Self;

    fn emit<E>(&mut self, event: E)
    where
        A: EventHandler<E>,
        E: Send + 'static;

    fn status(&self) -> &A::Status;
    fn set_status(&mut self, status: A::Status);
    fn update(&mut self) -> Result<(), <Self::Updater as Updater<A>>::Error>;

    fn controller(&self) -> &Self::Controller;
    fn sender(&self) -> &Self::Sender;

    fn updated_ref(&mut self) -> Option<&mut <Self::Updater as Updater<A>>::Updated>;
    fn updated(&mut self) -> Option<<Self::Updater as Updater<A>>::Updated>;

    fn controlled(&mut self) -> &mut <Self::Controller as Controller<A>>::Controlled;
    fn receiver(&mut self) -> &mut <Self::Sender as Sender<A>>::Receiver;
    fn updater(&mut self) -> &mut Self::Updater;
}

pub enum Work<A: Actor> {
    Action(Box<Action<Actor = A>>),
    Event(Box<Event<Actor = A>>),
    Message(Box<Message<Actor = A>>),
    Update,
}
