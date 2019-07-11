use std::future::Future;

use futures_core::Stream;
use futures_io as io;
use futures_io::AsyncRead;
use futures_io::AsyncWrite;

use crate::action::Action;
use crate::actor::Actor;
use crate::channel::Sender;
use crate::control::Controller;
use crate::event::Event;
use crate::event::EventHandler;
use crate::message::Handler;
use crate::message::Message;
use crate::spawned::Spawned;
use crate::update::Updater;

pub trait Context<A: Actor>: Stream<Item = Work<A>> + Unpin + Send {
    type Controller: Controller<A>;
    type Sender: Sender<A>;
    type Updater: Updater<A>;

    // TODO
    fn new(actor_id: u64) -> Self;

    /// Emits an event that will be handled by the
    /// actor.
    fn emit<E>(&mut self, event: E)
    where
        A: EventHandler<E>,
        E: Send + 'static;

    /// Gets the actor's current status.
    fn status(&self) -> &A::Status;

    /// Sets the actor's current status, eventually
    /// making action to reflect the change (e.g. by
    /// stopping the actor).
    fn set_status(&mut self, status: A::Status);

    /// Shares the actor's current status through
    /// the actor's update channel.
    fn update(&mut self) -> Result<(), <Self::Updater as Updater<A>>::Error>;

    /// Gets the actor's action channel sender.
    fn controller(&self) -> &Self::Controller;

    /// Gets the actor's message channel sender.
    fn sender(&self) -> &Self::Sender;

    /// Tries to get a mutable reference to the
    /// actor's update channel sender.
    fn updated_ref(&mut self) -> Option<&mut <Self::Updater as Updater<A>>::Updated>;

    /// Tries to get the actor's update channel
    /// sender.
    fn updated(&mut self) -> Option<<Self::Updater as Updater<A>>::Updated>;

    /// Gets a mutable reference to the actor's
    /// action channel receiver.
    fn controlled(&mut self) -> &mut <Self::Controller as Controller<A>>::Controlled;

    /// Gets a mutable reference to the actor's
    /// message channel receiver.
    fn receiver(&mut self) -> &mut <Self::Sender as Sender<A>>::Receiver;

    /// Gets a mutable reference to the actors's
    /// update channel receiver.
    fn updater(&mut self) -> &mut Self::Updater;

    // TODO
    fn actors(&self) -> Vec<u64>;

    // TODO
    fn spawn<S>(&mut self, actor: S) -> Option<Spawned<S>>
    where
        S: Actor + 'static;

    // TODO
    fn wait<F, M, O, T>(&mut self, fut: F, map: M)
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Send + 'static,
        A: Handler<T, Output = ()>,
        T: Send + 'static;

    // TODO
    fn subscribe<S, M, I, T>(&mut self, stream: S, map: M)
    where
        S: Stream<Item = I> + Unpin + Send + 'static,
        M: Fn(I) -> T + Send + 'static,
        A: Handler<T, Output = ()>,
        T: Send + 'static;

    // TODO
    fn read<R, M, N, T, E>(&mut self, read: R, cap: usize, map: M, map_err: N)
    where
        R: AsyncRead + Unpin + Send + 'static,
        M: Fn(Vec<u8>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()> + Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static;

    // TODO
    fn write<W, M, N, T, E>(&mut self, write: W, data: Vec<u8>, map: M, map_err: N)
    where
        W: AsyncWrite + Unpin + Send + 'static,
        M: Fn((Vec<u8>, usize), W) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()> + Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static;
}

pub enum Work<A: Actor> {
    /// Contains an action that should be handled
    /// by the actor.
    Action(Box<dyn Action<Actor = A>>),

    /// Contains an event that should be handled
    /// by the actor.
    Event(Box<dyn Event<Actor = A>>),

    /// Contains a message that should be handled
    /// by the actor.
    Message(Box<dyn Message<Actor = A>>),

    /// Indicates that the actor's status has
    /// changed.
    Update,
}
