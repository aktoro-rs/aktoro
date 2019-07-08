use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_channel::error::TrySendError;
use aktoro_raw as raw;
use aktoro_raw::Updater as RawUpdater;
use futures_core::Stream;
use futures_io::AsyncRead;
use futures_util::FutureExt;
use futures_util::StreamExt;

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
use crate::update;
use crate::update::Updated;
use crate::update::Updater;

/// An actor context using the [`aktoro-channel`] crate.
///
/// [`aktoro-channel`]: https://docs.rs/aktoro-channel
pub struct Context<A: raw::Actor> {
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
    futs: Vec<Pin<Box<dyn raw::AsyncMessageFut<Actor = A>>>>,
    // TODO
    streams: Vec<Pin<Box<dyn raw::AsyncMessageStream<Actor = A>>>>,
    // TODO
    reads: Vec<Pin<Box<dyn raw::AsyncReadStream<Actor = A>>>>,
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

impl<A> raw::Context<A> for Context<A>
where
    A: raw::Actor,
{
    type Controller = Controller<A>;
    type Sender = Sender<A>;
    type Updater = Updater<A>;

    fn new() -> Context<A> {
        // We create the actor's control, message and
        // update channels.
        let (ctrler, ctrled) = control::new();
        let (sender, recver) = channel::new();
        let (updter, updted) = update::new();

        Context {
            status: A::Status::default(),
            ctrler,
            ctrled,
            update: false,
            futs: Vec::new(),
            streams: Vec::new(),
            reads: Vec::new(),
            events: VecDeque::new(),
            sender,
            recver,
            updter,
            updted: Some(updted),
        }
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

    fn update(&mut self) -> Result<(), TrySendError<A::Status>> {
        self.update = false;
        self.updter.try_send(self.status.clone())
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

    fn wait<F, M, O, T>(&mut self, fut: F, map: M)
    where
        F: Future<Output = O> + Unpin + Send + 'static,
        M: Fn(O) -> T + Send + 'static,
        A: raw::Handler<T, Output = ()>,
        T: Send + 'static,
    {
        self.futs.push(Box::pin(
            AsyncMessageFut::new(fut.map(map)),
        ));
    }

    fn subscribe<S, M, I, T>(&mut self, stream: S, map: M)
    where
        S: Stream<Item = I> + Unpin + Send + 'static,
        M: Fn(I) -> T + Send + 'static,
        A: raw::Handler<T, Output = ()>,
        T: Send + 'static,
    {
        self.streams.push(Box::pin(
            AsyncMessageStream::new(stream.map(map)),
        ));
    }

    fn read<R, M, T>(&mut self, read: R, map: M)
    where
        R: AsyncRead + Unpin + Send + 'static,
        M: Fn(&mut [u8], usize) -> T + Unpin + Send + Sync + 'static,
        A: raw::Handler<T, Output = ()>,
        T: Send + 'static,
    {
        self.reads.push(Box::pin(
            AsyncReadStream::new(read, map),
        ));
    }
}

impl<A> Stream for Context<A>
where
    A: raw::Actor,
{
    type Item = raw::Work<A>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<raw::Work<A>>> {
        let context = self.get_mut();

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
        for (i, fut) in context.futs.iter_mut().enumerate() {
            match fut.as_mut().poll(ctx) {
                Poll::Ready(msg) => {
                    context.futs.remove(i);

                    return Poll::Ready(Some(raw::Work::Message(msg)))
                }
                Poll::Pending => (),
            }
        }

        // TODO
        let mut to_remove = vec![];
        for (i, stream) in context.streams.iter_mut().enumerate() {
            match stream.as_mut().poll_next(ctx) {
                Poll::Ready(Some(msg)) => {
                    return Poll::Ready(Some(raw::Work::Message(msg)));
                }
                Poll::Ready(None) => to_remove.push(i),
                Poll::Pending => (),
            }
        }

        for to_remove in to_remove {
            context.streams.remove(to_remove);
        }

        // TODO
        let mut to_remove = vec![];
        for (i, read) in context.reads.iter_mut().enumerate() {
            match read.as_mut().poll_read(ctx) {
                Poll::Ready(Ok(msg)) => {
                    return Poll::Ready(Some(raw::Work::Message(msg)));
                }
                Poll::Ready(Err(_)) => to_remove.push(i), // FIXME: handle error
                Poll::Pending => (),
            }
        }

        for to_remove in to_remove {
            context.reads.remove(to_remove);
        }

        Poll::Pending
    }
}
