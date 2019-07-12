use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task;
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

/// A wrapper around a future/stream/reader
/// that is returned by a context after asking
/// it to wait/subscribe/read, to allow to
/// cancel the action.
pub struct Cancellable<C>(CancellableInner<C>);

/// The structure that is actually holding
/// what [`Cancellable`] needs. It is also
/// what contexts will have to store and
/// update.
///
/// [`Cancellable`]: struct.Cancellable.html
pub struct CancellableInner<C> {
    /// The future/stream/reader that
    /// is holded.
    inner: Arc<AtomicCell<Option<Pin<Box<C>>>>>,
    /// Whether the action(s) are done or
    /// what is holded can be released.
    done: Arc<AtomicBool>,
    /// A reference to the space that
    /// was assigned to store the
    /// [`Cancelling`]'s waker, to wake it
    /// up when needed.
    ///
    /// [`Cancelling`]: struct.Cancelling.html
    waker: Arc<AtomicCell<Option<Waker>>>,
}

/// A future returned by [`Cancellable::cancel`]
/// that resolves when what is handled has
/// been cancelled (in which case it returns
/// it) or the action(s) are done.
pub struct Cancelling<C> {
    /// The future/stream/reader that is
    /// holded.
    inner: Arc<AtomicCell<Option<Pin<Box<C>>>>>,
    /// Whether the action(s) are done or
    /// what is holded can be released.
    done: Arc<AtomicBool>,
    /// A reference to the space that
    /// was assigned to store the
    /// [`Cancelling`]'s waker, to wake it
    /// up when needed.
    ///
    /// [`Cancelling`]: struct.Cancelling.html
    waker: Arc<AtomicCell<Option<Waker>>>,
}

pub trait Context<A: Actor>: Stream<Item = Work<A>> + Unpin + Send + Sized {
    type Config: Default;

    type Controller: Controller<A>;
    type Sender: Sender<A>;
    type Updater: Updater<A>;

    /// Creates a new context with the provided
    /// config and an identifier for the actor.
    fn new(actor_id: u64, config: Self::Config) -> Self;

    /// Returns the actor's identifier.
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

    /// Returns a list of the context's inner
    /// runtime's actors' identifier.
    fn actors(&self) -> Vec<u64>;

    /// Spawns a sub-actor on the context's inner
    /// runtime.
    ///
    /// ## Note
    ///
    /// The new actor must have a context with the
    /// same configuration structure as this context.
    fn spawn<S, C>(&mut self, actor: S) -> Option<Spawned<S>>
    where
        S: Actor<Context = C> + 'static,
        C: Context<S, Config = Self::Config>;

    /// Waits for a future to yield before mapping it
    /// to a message and passing it to the actor.
    ///
    /// The execution can be cancelled using the
    /// returned [`Cancellable`]. Cancelling the
    /// execution, if it isn't done, will return
    /// the original `fut`.
    ///
    /// [`Cancellable`]: struct.Cancellable.html
    fn wait<F, M, O, T>(&mut self, fut: Pin<Box<F>>, map: M) -> Cancellable<F>
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()>,
        O: Send + 'static,
        T: Send + 'static;

    /// Waits for a future to yield before mapping it
    /// to a message and passing it to the actor.
    ///
    /// Until all the blocking futures/asynchronous writes
    /// have yielded, no messages, events, streams, etc.
    /// will be handled by the context.
    ///
    /// The execution can be cancelled using the
    /// returned [`Cancellable`]. Cancelling the
    /// execution, if it isn't done, will return
    /// the original `fut`.
    ///
    /// [`Cancellable`]: struct.Cancellable.html
    fn blocking_wait<F, M, O, T>(&mut self, fut: Pin<Box<F>>, map: M) -> Cancellable<F>
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()>,
        O: Send + 'static,
        T: Send + 'static;

    /// Forwards the items yielded by a stream to
    /// the actor after mapping them to a message.
    ///
    /// The execution can be cancelled using the
    /// returned [`Cancellable`]. Cancelling the
    /// execution, if it isn't done, will return
    /// the original `fut`.
    ///
    /// [`Cancellable`]: struct.Cancellable.html
    fn subscribe<S, M, I, T>(&mut self, stream: Pin<Box<S>>, map: M) -> Cancellable<S>
    where
        S: Stream<Item = I> + Unpin + Send + 'static,
        M: Fn(I) -> T + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()>,
        I: Send + 'static,
        T: Send + 'static;

    /// Forwards the received data to the actor
    /// after either mapping it or a returned
    /// error to a message.
    ///
    /// The execution can be cancelled using the
    /// returned [`Cancellable`]. Cancelling the
    /// execution, if it isn't done, will return
    /// the original `fut`.
    ///
    /// [`Cancellable`]: struct.Cancellable.html
    fn read<R, M, N, T, E>(
        &mut self,
        read: Pin<Box<R>>,
        cap: usize,
        map: M,
        map_err: N,
    ) -> Cancellable<R>
    where
        R: AsyncRead + Unpin + Send + 'static,
        M: Fn(Vec<u8>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()> + Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static;

    /// Waits for data to be written over an asynchronous
    /// writer, then passing a message returned by either
    /// `map` or `map_err` (depending on whether an error
    /// was returned by the writer) to the actor.
    ///
    /// The execution can be cancelled using the
    /// returned [`Cancellable`]. Cancelling the
    /// execution, if it isn't done, will return
    /// the original `fut`.
    ///
    /// [`Cancellable`]: struct.Cancellable.html
    fn write<W, M, N, T, E>(
        &mut self,
        write: Pin<Box<W>>,
        data: Vec<u8>,
        map: M,
        map_err: N,
    ) -> Cancellable<W>
    where
        W: AsyncWrite + Unpin + Send + 'static,
        M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: Handler<T, Output = ()> + Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static;

    /// Waits for data to be written over an asynchronous
    /// writer, then passing a message returned by either
    /// `map` or `map_err` (depending on whether an error
    /// was returned by the writer) to the actor.
    ///
    /// Until all the blocking futures/asynchronous writes
    /// have yielded, no messages, events, streams, etc.
    /// will be handled by the context.
    ///
    /// The execution can be cancelled using the
    /// returned [`Cancellable`]. Cancelling the
    /// execution, if it isn't done, will return
    /// the original `fut`.
    ///
    /// [`Cancellable`]: struct.Cancellable.html
    fn blocking_write<W, M, N, T, E>(
        &mut self,
        write: Pin<Box<W>>,
        data: Vec<u8>,
        map: M,
        map_err: N,
    ) -> Cancellable<W>
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
    pub fn new(inner: Pin<Box<C>>) -> (Self, CancellableInner<C>) {
        let inner = Arc::new(AtomicCell::new(Some(inner)));
        let done = Arc::new(AtomicBool::new(false));
        let waker = Arc::new(AtomicCell::new(None));

        (
            Cancellable(CancellableInner {
                inner: inner.clone(),
                done: done.clone(),
                waker: waker.clone(),
            }),
            CancellableInner { inner, done, waker },
        )
    }

    /// Creates a future that will yield when
    /// the action(s) have either been cancelled,
    /// in which case it will also give back what
    /// was holded, or are done.
    pub fn cancel(self) -> Cancelling<C> {
        Cancelling {
            inner: self.0.inner,
            done: self.0.done,
            waker: self.0.waker,
        }
    }
}

impl<C> CancellableInner<C> {
    /// Gets what the wrapper holds.
    ///
    /// ## Note
    ///
    /// If the action(s) aren't done, you need
    /// to call [`set`].
    ///
    /// [`set`]: #method.set
    pub fn get(&self) -> Option<Pin<Box<C>>> {
        self.inner.swap(None)
    }

    /// Sets what the wrapper holds to `inner`.
    pub fn set(&self, inner: Pin<Box<C>>) {
        self.inner.store(Some(inner));

        // We eventually wake up the future
        // that wants to get what's holded back.
        if let Some(waker) = self.waker.swap(None) {
            waker.wake();
        }
    }

    /// Sets the action(s) as done.
    pub fn done(&self) {
        self.inner.store(None);
        self.done.store(true, Ordering::SeqCst);

        // We eventually wake up the future
        // that wanted to get what's holded back,
        // so that it can know that it wont be
        // able to.
        if let Some(waker) = self.waker.swap(None) {
            waker.wake();
        }
    }
}

impl<C> Future for Cancelling<C> {
    type Output = Option<Pin<Box<C>>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Option<Pin<Box<C>>>> {
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
