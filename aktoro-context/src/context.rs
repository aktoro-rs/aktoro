use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task;
use std::task::Poll;

use aktoro_channel::error::TrySendError;
use aktoro_channel::Notify;
use aktoro_raw as raw;
use aktoro_raw::Updater as RawUpdater;
use aktoro_raw::Wait as RawWait;
use futures_core::Stream;
use futures_io as io;
use futures_io::AsyncRead;
use futures_io::AsyncWrite;

use crate::channel;
use crate::channel::Receiver;
use crate::channel::Sender;
use crate::control;
use crate::control::Controlled;
use crate::control::Controller;
use crate::event::Event;
use crate::message::AsyncMessageFut;
use crate::message::AsyncMessageStream;
use crate::message::AsyncReadStream;
use crate::message::AsyncWriteFut;
use crate::update;
use crate::update::Update;
use crate::update::Updated;
use crate::update::Updater;

// TODO
pub struct ContextConfig {
    ready: Option<Notify>,
}

/// An actor context using the [`aktoro-channel`] crate.
///
/// [`aktoro-channel`]: https://docs.rs/aktoro-channel
pub struct Context<A: raw::Actor, R: raw::Runtime> {
    // TODO
    actor_id: u64,
    // TODO
    ready: Option<Notify>,
    /// The actor's current status.
    status: A::Status,
    /// An actor's control channel sender.
    ctrler: Controller<A>,
    /// An actor's control channel receiver.
    ctrled: Controlled<A>,
    /// Whether the status has been recently updated
    /// and the runtime should be notified.
    update: bool,
    // TODO
    b_futs: Vec<Pin<Box<dyn raw::AsyncMessageFut<Actor = A>>>>,
    // TODO
    futs: Vec<Pin<Box<dyn raw::AsyncMessageFut<Actor = A>>>>,
    // TODO
    streams: Vec<Pin<Box<dyn raw::AsyncMessageStream<Actor = A>>>>,
    // TODO
    reads: Vec<Pin<Box<dyn raw::AsyncReadStream<Actor = A>>>>,
    // TODO
    rt: Option<R>,
    // TODO
    to_notify: Vec<Notify>,
    /// A list of the actor's unhandled events.
    events: VecDeque<Box<dyn raw::Event<Actor = A>>>,
    /// An actor's message channel sender.
    sender: Sender<A>,
    /// An actor's message channel receiver.
    recver: Receiver<A>,
    /// An actor's update channel sender.
    updter: Updater<A>,
    /// An actor's update channel receiver.
    updted: Option<Updated<A>>,
}

impl<A, RT> raw::Context<A> for Context<A, RT>
where
    A: raw::Actor + 'static,
    RT: raw::Runtime,
{
    type Config = ContextConfig;

    type Controller = Controller<A>;
    type Sender = Sender<A>;
    type Updater = Updater<A>;

    fn new(actor_id: u64, config: ContextConfig) -> Context<A, RT> {
        // We create the actor's control, message and
        // update channels.
        let (ctrler, ctrled) = control::new();
        let (sender, recver) = channel::new();
        let (updter, updted) = update::new();

        Context {
            actor_id,
            ready: config.ready,
            status: A::Status::default(),
            ctrler,
            ctrled,
            update: false,
            b_futs: vec![],
            futs: vec![],
            streams: vec![],
            reads: vec![],
            rt: None,
            to_notify: vec![],
            events: VecDeque::new(),
            sender,
            recver,
            updter,
            updted: Some(updted),
        }
    }

    fn actor_id(&self) -> u64 {
        self.actor_id
    }

    fn emit<E>(&mut self, event: E)
    where
        A: raw::EventHandler<E>,
        E: Send + 'static,
    {
        self.events.push_back(Box::new(Event::new(event)));
    }

    fn status(&self) -> &A::Status {
        &self.status
    }

    fn set_status(&mut self, status: A::Status) {
        if self.status != status {
            self.status = status;
            self.update = true;
        }
    }

    fn update(&mut self) -> Result<(), TrySendError<Update<A>>> {
        self.update = false;
        self.updter
            .try_send(Update::new(self.actor_id, self.status.clone()))
    }

    fn controller(&self) -> &Controller<A> {
        &self.ctrler
    }

    fn sender(&self) -> &Sender<A> {
        &self.sender
    }

    fn updated_ref(&mut self) -> Option<&mut Updated<A>> {
        self.updted.as_mut()
    }

    fn updated(&mut self) -> Option<Updated<A>> {
        self.updted.take()
    }

    fn controlled(&mut self) -> &mut Controlled<A> {
        &mut self.ctrled
    }

    fn receiver(&mut self) -> &mut Receiver<A> {
        &mut self.recver
    }

    fn updater(&mut self) -> &mut Updater<A> {
        &mut self.updter
    }

    fn actors(&self) -> Vec<u64> {
        if let Some(rt) = &self.rt {
            rt.actors()
        } else {
            vec![]
        }
    }

    fn spawn<S, C>(&mut self, actor: S) -> Option<raw::Spawned<S>>
    where
        S: raw::Actor<Context = C> + 'static,
        C: raw::Context<S, Config = ContextConfig>,
    {
        let rt = if let Some(rt) = &mut self.rt {
            rt
        } else {
            self.rt = Some(RT::default());
            self.rt.as_mut().unwrap()
        };

        let mut config = ContextConfig::default();

        let (notify, ready) = Notify::new();
        config.ready = Some(ready);

        if let Some(spawned) = rt.spawn_with(actor, config) {
            self.to_notify.push(notify);
            Some(spawned)
        } else {
            None
        }
    }

    fn wait<F, M, O, T>(&mut self, fut: Pin<Box<F>>, map: M) -> raw::Cancellable<F>
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()>,
        O: Send + 'static,
        T: Send + 'static,
    {
        let (cancellable, inner) = raw::Cancellable::new(fut);

        self.futs.push(Box::pin(AsyncMessageFut::new(inner, map)));

        cancellable
    }

    fn blocking_wait<F, M, O, T>(&mut self, fut: Pin<Box<F>>, map: M) -> raw::Cancellable<F>
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()>,
        O: Send + 'static,
        T: Send + 'static,
    {
        let (cancellable, inner) = raw::Cancellable::new(fut);
        self.b_futs.push(Box::pin(AsyncMessageFut::new(inner, map)));

        cancellable
    }

    fn subscribe<S, M, I, T>(&mut self, stream: Pin<Box<S>>, map: M) -> raw::Cancellable<S>
    where
        S: Stream<Item = I> + Unpin + Send + 'static,
        M: Fn(I) -> T + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()>,
        I: Send + 'static,
        T: Send + 'static,
    {
        let (cancellable, inner) = raw::Cancellable::new(stream);

        self.streams
            .push(Box::pin(AsyncMessageStream::new(inner, map)));

        cancellable
    }

    fn read<R, M, N, T, E>(
        &mut self,
        read: Pin<Box<R>>,
        cap: usize,
        map: M,
        map_err: N,
    ) -> raw::Cancellable<R>
    where
        R: AsyncRead + Unpin + Send + 'static,
        M: Fn(Vec<u8>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static,
    {
        let (cancellable, inner) = raw::Cancellable::new(read);

        self.reads
            .push(Box::pin(AsyncReadStream::new(inner, cap, map, map_err)));

        cancellable
    }

    fn write<W, M, N, T, E>(
        &mut self,
        write: Pin<Box<W>>,
        data: Vec<u8>,
        map: M,
        map_err: N,
    ) -> raw::Cancellable<W>
    where
        W: AsyncWrite + Unpin + Send + 'static,
        M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static,
    {
        let (cancellable, inner) = raw::Cancellable::new(write);

        self.futs
            .push(Box::pin(AsyncWriteFut::new(inner, data, map, map_err)));

        cancellable
    }

    fn blocking_write<W, M, N, T, E>(
        &mut self,
        write: Pin<Box<W>>,
        data: Vec<u8>,
        map: M,
        map_err: N,
    ) -> raw::Cancellable<W>
    where
        W: AsyncWrite + Unpin + Send + 'static,
        M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync + 'static,
        N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
        T: Send + 'static,
        E: Send + 'static,
    {
        let (cancellable, inner) = raw::Cancellable::new(write);

        self.b_futs
            .push(Box::pin(AsyncWriteFut::new(inner, data, map, map_err)));

        cancellable
    }
}

impl<A, R> Stream for Context<A, R>
where
    A: raw::Actor,
    R: raw::Runtime,
{
    type Item = raw::Work<A>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Option<raw::Work<A>>> {
        let context = self.get_mut();
        let mut ret = None;

        if let Some(ready) = context.ready.as_mut() {
            match Pin::new(ready).poll(ctx) {
                Poll::Ready(()) => {
                    context.ready.take();
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        // If an action has been received an action has
        // been received from the actor's control channel,
        // we ask the runtime to make the actor handle it.
        match Pin::new(&mut context.ctrled).poll_next(ctx) {
            Poll::Ready(Some(update)) => {
                return Poll::Ready(Some(raw::Work::Action(update)));
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => (),
        }

        // If the actor's status has been recently updated,
        // we notify the runtime.
        if context.update {
            context.update = false;
            return Poll::Ready(Some(raw::Work::Update));
        }

        // TODO
        let mut to_remove = vec![];
        for (i, fut) in context.b_futs.iter_mut().enumerate() {
            match fut.as_mut().poll(ctx) {
                Poll::Ready(Some(msg)) => {
                    to_remove.push(i);
                    ret = Some(raw::Work::Message(msg));

                    break;
                }
                Poll::Ready(None) => to_remove.push(i),
                Poll::Pending => (),
            }
        }

        for to_remove in to_remove {
            context.b_futs.remove(to_remove);
        }

        if let Some(ret) = ret {
            return Poll::Ready(Some(ret));
        } else if !context.b_futs.is_empty() {
            return Poll::Pending;
        }

        // TODO
        for to_notify in context.to_notify.drain(..) {
            to_notify.done();
        }

        // If the actor has unhandled events, we ask the
        // runtime to make the actor handle the oldest
        // one.
        if let Some(event) = context.events.pop_front() {
            return Poll::Ready(Some(raw::Work::Event(event)));
        }

        // If a message has been received from the actor's
        // message channel, we ask the runtime to make
        // the runtime handle it.
        match Pin::new(&mut context.recver).poll_next(ctx) {
            Poll::Ready(Some(msg)) => {
                return Poll::Ready(Some(raw::Work::Message(msg)));
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => (),
        }

        // TODO
        if let Some(rt) = context.rt.take() {
            let mut wait = rt.wait();

            loop {
                if wait.runtime().actors().is_empty() {
                    break;
                }

                if let Poll::Pending = Pin::new(&mut wait).poll_next(ctx) {
                    break;
                }
            }

            context.rt = Some(wait.into_runtime());
        }

        // TODO
        let mut to_remove = vec![];
        for (i, fut) in context.futs.iter_mut().enumerate() {
            match fut.as_mut().poll(ctx) {
                Poll::Ready(Some(msg)) => {
                    to_remove.push(i);
                    ret = Some(raw::Work::Message(msg));

                    break;
                }
                Poll::Ready(None) => to_remove.push(i),
                Poll::Pending => (),
            }
        }

        for to_remove in to_remove {
            context.futs.remove(to_remove);
        }

        if let Some(ret) = ret {
            return Poll::Ready(Some(ret));
        }

        // TODO
        let mut to_remove = vec![];
        for (i, stream) in context.streams.iter_mut().enumerate() {
            match stream.as_mut().poll_next(ctx) {
                Poll::Ready(Some(msg)) => ret = Some(raw::Work::Message(msg)),
                Poll::Ready(None) => to_remove.push(i),
                Poll::Pending => (),
            }
        }

        for to_remove in to_remove {
            context.streams.remove(to_remove);
        }

        if let Some(ret) = ret {
            return Poll::Ready(Some(ret));
        }

        // TODO
        let mut to_remove = vec![];
        for (i, read) in context.reads.iter_mut().enumerate() {
            match read.as_mut().poll_read(ctx) {
                Poll::Ready(Some(msg)) => ret = Some(raw::Work::Message(msg)),
                Poll::Ready(None) => to_remove.push(i),
                Poll::Pending => (),
            }
        }

        for to_remove in to_remove {
            context.reads.remove(to_remove);
        }

        if let Some(ret) = ret {
            return Poll::Ready(Some(ret));
        }

        Poll::Pending
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        ContextConfig { ready: None }
    }
}
