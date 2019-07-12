use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::Context as FutContext;
use std::task::Poll;
use std::task::Waker;

use crossbeam_utils::atomic::AtomicCell;
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

// TODO
pub struct Cancellable<C> {
    // TODO
    inner: CancellableInner<C>,
}

// TODO
pub struct CancellableInner<C> {
    // TODO
    inner: Arc<AtomicCell<Option<Pin<Box<C>>>>>,
    // TODO
    done: Arc<AtomicBool>,
    // TODO
    waker: Arc<AtomicCell<Option<Waker>>>,
}

// TODO
pub struct Cancelling<C> {
    // TODO
    inner: Arc<AtomicCell<Option<Pin<Box<C>>>>>,
    // TODO
    done: Arc<AtomicBool>,
    // TODO
    waker: Arc<AtomicCell<Option<Waker>>>,
}

pub trait Context<A: Actor>: Stream<Item = Work<A>> + Unpin + Send + Sized {
    type Config: Default;

    type Controller: Controller<A>;
    type Sender: Sender<A>;
    type Updater: Updater<A>;

    // TODO
    fn new(actor_id: u64, config: Self::Config) -> Self;

    // TODO
    fn actor_id(&self) -> u64;

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
    fn spawn<S, C>(&mut self, actor: S) -> Option<Spawned<S>>
    where
        S: Actor<Context = C> + 'static,
        C: Context<S, Config = Self::Config>;

    // TODO
    fn wait<F, M, O, T>(&mut self, fut: Pin<Box<F>>, map: M) -> Cancellable<F>
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()>,
        O: Send + 'static,
        T: Send + 'static;

    // TODO
    fn blocking_wait<F, M, O, T>(&mut self, fut: Pin<Box<F>>, map: M) -> Cancellable<F>
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()>,
        O: Send + 'static,
        T: Send + 'static;

    // TODO
    fn subscribe<S, M, I, T>(&mut self, stream: Pin<Box<S>>, map: M) -> Cancellable<S>
    where
        S: Stream<Item = I> + Unpin + Send + 'static,
        M: Fn(I) -> T + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()>,
        I: Send + 'static,
        T: Send + 'static;

    // TODO
    fn read<R, M, N, T, E>(&mut self, read: Pin<Box<R>>, cap: usize, map: M, map_err: N) -> Cancellable<R>
    where
        R: AsyncRead + Unpin + Send + 'static,
        M: Fn(Vec<u8>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()> + Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static;

    // TODO
    fn write<W, M, N, T, E>(&mut self, write: Pin<Box<W>>, data: Vec<u8>, map: M, map_err: N) -> Cancellable<W>
    where
        W: AsyncWrite + Unpin + Send + 'static,
        M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()> + Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static;

    // TODO
    fn blocking_write<W, M, N, T, E>(&mut self, write: Pin<Box<W>>, data: Vec<u8>, map: M, map_err: N) -> Cancellable<W>
    where
        W: AsyncWrite + Unpin + Send + 'static,
        M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync + 'static,
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

impl<C> Cancellable<C> {
    // TODO
    pub fn new(inner: Pin<Box<C>>) -> (Self, CancellableInner<C>) {
        let inner = Arc::new(AtomicCell::new(Some(inner)));
        let done = Arc::new(AtomicBool::new(false));
        let waker = Arc::new(AtomicCell::new(None));

        (
            Cancellable {
                inner: CancellableInner {
                    inner: inner.clone(),
                    done: done.clone(),
                    waker: waker.clone(),
                }
            },
            CancellableInner { inner, done, waker, },
        )
    }

    // TODO
    pub fn cancel(self) -> Cancelling<C> {
        Cancelling {
            inner: self.inner.inner,
            done: self.inner.done,
            waker: self.inner.waker,
        }
    }
}

impl<C> CancellableInner<C> {
    // TODO
    pub fn get(&self) -> Option<Pin<Box<C>>> {
        self.inner.swap(None)
    }

    // TODO
    pub fn set(&self, inner: Pin<Box<C>>) {
        self.inner.store(Some(inner));

        if let Some(waker) = self.waker.swap(None) {
            waker.wake();
        }
    }

    // TODO
    pub fn done(&self) {
        self.inner.store(None);
        self.done.store(true, Ordering::SeqCst);

        if let Some(waker) = self.waker.swap(None) {
            waker.wake();
        }
    }
}

impl<C> Future for Cancelling<C> {
    type Output = Option<Pin<Box<C>>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<Pin<Box<C>>>> {
        if self.done.load(Ordering::SeqCst) {
            return Poll::Ready(None);
        }

        match self.inner.swap(None) {
            Some(inner) => {
                self.waker.store(None);

                Poll::Ready(Some(inner))
            }
            None => {
                self.waker.store(Some(ctx.waker().clone()));

                Poll::Pending
            }
        }
    }
}
