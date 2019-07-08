use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_raw as raw;
use futures_core::Stream;
use futures_io::AsyncRead;
use futures_io::Error as FutIOError;

use crate::respond::Respond;

/// A wrapper around a message that an actor should
/// handle (this is used to allow generalization).
pub(crate) struct Message<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    msg: Option<M>,
    resp: Option<Respond<A::Output>>,
}

pub(crate) struct AsyncMessage<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    msg: Option<M>,
    _act: PhantomData<A>,
}

pub(crate) struct AsyncMessageFut<A, F, M>
where
    A: raw::Handler<M, Output = ()>,
    F: Future<Output = M> + Unpin + Send,
    M: Send,
{
    fut: F,
    _act: PhantomData<A>,
}

pub(crate) struct AsyncMessageStream<A, S, M>
where
    A: raw::Handler<M, Output = ()>,
    S: Stream<Item = M> + Unpin + Send,
    M: Send,
{
    stream: S,
    _act: PhantomData<A>,
}

pub(crate) struct AsyncReadStream<A, R, M, T>
where
    A: raw::Handler<T, Output = ()>,
    R: AsyncRead + Unpin + Send,
    M: Fn(&mut [u8], usize) -> T + Send,
    T: Send,
{
    buf: Vec<u8>,
    read: R,
    map: M,
    _act: PhantomData<A>,
}

impl<A, M> Message<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    pub(crate) fn new(msg: M) -> (Self, Respond<A::Output>) {
        let resp = Respond::new();

        (
            Message {
                msg: Some(msg),
                resp: Some(resp.0),
            },
            resp.1,
        )
    }
}

impl<A, M> AsyncMessage<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    pub(crate) fn new(msg: M) -> Self {
        AsyncMessage {
            msg: Some(msg),
            _act: PhantomData,
        }
    }
}

impl<A, F, M> AsyncMessageFut<A, F, M>
where
    A: raw::Handler<M, Output = ()>,
    F: Future<Output = M> + Unpin + Send,
    M: Send,
{
    pub(crate) fn new(fut: F) -> Self {
        AsyncMessageFut {
            fut,
            _act: PhantomData,
        }
    }
}

impl<A, S, M> AsyncMessageStream<A, S, M>
where
    A: raw::Handler<M, Output = ()>,
    S: Stream<Item = M> + Unpin + Send,
    M: Send,
{
    pub(crate) fn new(stream: S) -> Self {
        AsyncMessageStream {
            stream,
            _act: PhantomData,
        }
    }
}

impl<A, R, M, T> AsyncReadStream<A, R, M, T>
where
    A: raw::Handler<T, Output = ()>,
    R: AsyncRead + Unpin + Send,
    M: Fn(&mut [u8], usize) -> T + Unpin + Send,
    T: Send,
{
    pub(crate) fn new(read: R, map: M) -> Self {
        AsyncReadStream {
            buf: Vec::new(),
            read,
            map,
            _act: PhantomData,
        }
    }
}

impl<A, M> raw::Message for Message<A, M>
where
    A: raw::Handler<M>,
    M: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) -> Result<(), A::Error> {
        // If the message hasn't already been handled,
        // we do so and return the result.
        if let Some(msg) = self.msg.take() {
            self.resp.take().unwrap().respond(actor.handle(msg, ctx)?);
        }

        Ok(())
    }
}

impl<A, M> raw::Message for AsyncMessage<A, M>
where
    A: raw::Handler<M, Output = ()>,
    M: Send,
{
    type Actor = A;

    fn handle(&mut self, actor: &mut A, ctx: &mut A::Context) -> Result<(), A::Error> {
        // If the message hasn't already been handled,
        // we do so and return the result.
        if let Some(msg) = self.msg.take() {
            actor.handle(msg, ctx)
        } else {
            Ok(())
        }
    }
}

impl<A, F, M> raw::AsyncMessageFut for AsyncMessageFut<A, F, M>
where
    A: raw::Handler<M, Output = ()>,
    F: Future<Output = M> + Unpin + Send,
    M: Send + 'static,
{
    type Actor = A;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Box<dyn raw::Message<Actor = A>>> {
        match Pin::new(&mut self.get_mut().fut).poll(ctx) {
            Poll::Ready(msg) => Poll::Ready(Box::new(AsyncMessage::new(msg))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A, S, M> raw::AsyncMessageStream for AsyncMessageStream<A, S, M>
where
    A: raw::Handler<M, Output = ()>,
    S: Stream<Item = M> + Unpin + Send,
    M: Send + 'static,
{
    type Actor = A;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<Option<Box<dyn raw::Message<Actor = A>>>> {
        match Pin::new(&mut self.get_mut().stream).poll_next(ctx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Some(Box::new(AsyncMessage::new(msg)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A, R, M, T> raw::AsyncReadStream for AsyncReadStream<A, R, M, T>
where
    A: raw::Handler<T, Output = ()>,
    R: AsyncRead + Unpin + Send,
    M: Fn(&mut [u8], usize) -> T + Unpin + Send,
    T: Send + 'static,
{
    type Actor = A;

    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<Result<Box<dyn raw::Message<Actor = A>>, FutIOError>> {
        let stream = self.get_mut();

        match Pin::new(&mut stream.read).poll_read(ctx, &mut stream.buf) {
            Poll::Ready(Ok(read)) => {
                let msg = (stream.map)(&mut stream.buf, read);

                Poll::Ready(Ok(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}
