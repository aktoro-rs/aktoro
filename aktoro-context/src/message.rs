use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_raw as raw;
use futures_core::Stream;
use futures_io as io;
use futures_io::AsyncRead;
use futures_io::AsyncWrite;

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

pub(crate) struct AsyncReadStream<A, R, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    R: AsyncRead + Unpin + Send,
    M: Fn(Vec<u8>) -> T + Send,
    N: Fn(io::Error) -> E + Send,
    T: Send,
    E: Send,
{
    cap: usize,
    buf: Vec<u8>,
    read: R,
    map: M,
    map_err: N,
    _act: PhantomData<A>,
}

pub(crate) struct AsyncWriteFut<A, W, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    W: AsyncWrite + Unpin + Send,
    M: Fn((Vec<u8>, usize), W) -> T + Unpin + Send + Sync + 'static,
    N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
    T: Send,
    E: Send,
{
    data: Option<Vec<u8>>,
    map: M,
    map_err: N,
    write: Option<W>,
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

impl<A, R, M, N, T, E> AsyncReadStream<A, R, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    R: AsyncRead + Unpin + Send,
    M: Fn(Vec<u8>) -> T + Unpin + Send,
    N: Fn(io::Error) -> E + Unpin + Send,
    T: Send,
    E: Send,
{
    pub(crate) fn new(read: R, cap: usize, map: M, map_err: N) -> Self {
        let mut buf = Vec::with_capacity(cap);
        buf.resize(cap, 0);

        AsyncReadStream {
            cap,
            buf,
            read,
            map,
            map_err,
            _act: PhantomData,
        }
    }
}

impl<A, W, M, N, T, E> AsyncWriteFut<A, W, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    W: AsyncWrite + Unpin + Send,
    M: Fn((Vec<u8>, usize), W)  -> T + Unpin + Send + Sync,
    N: Fn(io::Error) -> E + Unpin + Send + Sync,
    T: Send,
    E: Send,
{
    pub(crate) fn new(write: W, data: Vec<u8>, map: M, map_err: N) -> Self {
        AsyncWriteFut {
            data: Some(data),
            map,
            map_err,
            write: Some(write),
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

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<raw::AsyncMessageOutput<Self::Actor>> {
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
    ) -> Poll<raw::AsyncMessageItem<Self::Actor>> {
        match Pin::new(&mut self.get_mut().stream).poll_next(ctx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Some(Box::new(AsyncMessage::new(msg)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A, R, M, N, T, E> raw::AsyncReadStream for AsyncReadStream<A, R, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    R: AsyncRead + Unpin + Send,
    M: Fn(Vec<u8>) -> T + Unpin + Send,
    N: Fn(io::Error) -> E + Unpin + Send,
    T: Send + 'static,
    E: Send + 'static,
{
    type Actor = A;

    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<raw::AsyncMessageOutput<Self::Actor>> {
        let stream = self.get_mut();

        match Pin::new(&mut stream.read).poll_read(ctx, &mut stream.buf) {
            Poll::Ready(Ok(read)) => {
                let data = stream.buf.drain(0..read).collect();
                stream.buf.resize(stream.cap, 0);

                let msg = (stream.map)(data);

                Poll::Ready(Box::new(AsyncMessage::new(msg)))
            }
            Poll::Ready(Err(err)) => {
                let msg = (stream.map_err)(err);

                Poll::Ready(Box::new(AsyncMessage::new(msg)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A, W, M, N, T, E> raw::AsyncMessageFut for AsyncWriteFut<A, W, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    W: AsyncWrite + Unpin + Send,
    M: Fn((Vec<u8>, usize), W) -> T + Unpin + Send + Sync,
    N: Fn(io::Error) -> E + Unpin + Send + Sync,
    T: Send + 'static,
    E: Send + 'static,
{
    type Actor = A;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
    ) -> Poll<raw::AsyncMessageOutput<Self::Actor>> {
        let fut = self.get_mut();

        match Pin::new(&mut fut.write.as_mut().unwrap())
            .poll_write(ctx, fut.data.as_ref().unwrap())
        {
            Poll::Ready(Ok(wrote)) => {
                let msg = (fut.map)(
                    (fut.data.take().unwrap(), wrote),
                    fut.write.take().unwrap(),
                );

                Poll::Ready(Box::new(AsyncMessage::new(msg)))
            }
            Poll::Ready(Err(err)) => {
                let msg = (fut.map_err)(err);

                Poll::Ready(Box::new(AsyncMessage::new(msg)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
