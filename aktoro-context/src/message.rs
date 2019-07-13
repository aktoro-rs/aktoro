use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task;
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

pub(crate) struct AsyncMessageFut<A, F, M, O, T>
where
    A: raw::Handler<T, Output = ()>,
    F: Future<Output = O> + Unpin + Send,
    M: Fn(O) -> T + Send,
    O: Send,
    T: Send,
{
    inner: raw::CancellableInner<F>,
    map: M,
    _act: PhantomData<A>,
}

pub(crate) struct AsyncMessageStream<A, S, M, I, T>
where
    A: raw::Handler<T, Output = ()>,
    S: Stream<Item = I> + Unpin + Send,
    M: Fn(I) -> T + Send,
    I: Send,
    T: Send,
{
    inner: raw::CancellableInner<S>,
    map: M,
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
    inner: raw::CancellableInner<R>,
    map: M,
    map_err: N,
    _act: PhantomData<A>,
}

pub(crate) struct AsyncWriteFut<A, W, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()>,
    W: AsyncWrite + Unpin + Send,
    M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync + 'static,
    N: Fn(io::Error) -> E + Unpin + Send + Sync + 'static,
    T: Send,
    E: Send,
{
    data: Option<Vec<u8>>,
    map: M,
    map_err: N,
    inner: raw::CancellableInner<W>,
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

impl<A, F, M, O, T> AsyncMessageFut<A, F, M, O, T>
where
    A: raw::Handler<T, Output = ()>,
    F: Future<Output = O> + Unpin + Send,
    M: Fn(O) -> T + Unpin + Send,
    O: Send,
    T: Send,
{
    pub(crate) fn new(inner: raw::CancellableInner<F>, map: M) -> Self {
        AsyncMessageFut {
            inner,
            map,
            _act: PhantomData,
        }
    }
}

impl<A, S, M, I, T> AsyncMessageStream<A, S, M, I, T>
where
    A: raw::Handler<T, Output = ()>,
    S: Stream<Item = I> + Unpin + Send,
    M: Fn(I) -> T + Unpin + Send,
    I: Send,
    T: Send,
{
    pub(crate) fn new(inner: raw::CancellableInner<S>, map: M) -> Self {
        AsyncMessageStream {
            inner,
            map,
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
    pub(crate) fn new(inner: raw::CancellableInner<R>, cap: usize, map: M, map_err: N) -> Self {
        AsyncReadStream {
            cap,
            buf: vec![0; cap],
            inner,
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
    M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync,
    N: Fn(io::Error) -> E + Unpin + Send + Sync,
    T: Send,
    E: Send,
{
    pub(crate) fn new(inner: raw::CancellableInner<W>, data: Vec<u8>, map: M, map_err: N) -> Self {
        AsyncWriteFut {
            data: Some(data),
            map,
            map_err,
            inner,
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

impl<A, F, M, O, T> raw::AsyncMessageFut for AsyncMessageFut<A, F, M, O, T>
where
    A: raw::Handler<T, Output = ()> + 'static,
    F: Future<Output = O> + Unpin + Send,
    M: Fn(O) -> T + Unpin + Send,
    O: Send + 'static,
    T: Send + 'static,
{
    type Actor = A;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<raw::AsyncMessageRet<Self::Actor>> {
        let fut = self.get_mut();
        let mut inner = if let Some(inner) = fut.inner.get() {
            inner
        } else {
            return Poll::Ready(None);
        };

        match Pin::new(&mut inner).poll(ctx) {
            Poll::Ready(output) => {
                let msg = (fut.map)(output);

                fut.inner.done();
                Poll::Ready(Some(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Pending => {
                fut.inner.set(inner);
                Poll::Pending
            }
        }
    }
}

impl<A, S, M, I, T> raw::AsyncMessageStream for AsyncMessageStream<A, S, M, I, T>
where
    A: raw::Handler<T, Output = ()> + 'static,
    S: Stream<Item = I> + Unpin + Send,
    M: Fn(I) -> T + Unpin + Send,
    I: Send + 'static,
    T: Send + 'static,
{
    type Actor = A;

    fn poll_next(
        self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<raw::AsyncMessageRet<Self::Actor>> {
        let stream = self.get_mut();
        let mut inner = if let Some(inner) = stream.inner.get() {
            inner
        } else {
            return Poll::Ready(None);
        };

        match Pin::new(&mut inner).poll_next(ctx) {
            Poll::Ready(Some(item)) => {
                let msg = (stream.map)(item);

                stream.inner.set(inner);
                Poll::Ready(Some(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Ready(None) => {
                stream.inner.done();
                Poll::Ready(None)
            }
            Poll::Pending => {
                stream.inner.set(inner);
                Poll::Pending
            }
        }
    }
}

impl<A, R, M, N, T, E> raw::AsyncReadStream for AsyncReadStream<A, R, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()> + 'static,
    R: AsyncRead + Unpin + Send,
    M: Fn(Vec<u8>) -> T + Unpin + Send,
    N: Fn(io::Error) -> E + Unpin + Send,
    T: Send + 'static,
    E: Send + 'static,
{
    type Actor = A;

    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<raw::AsyncMessageRet<Self::Actor>> {
        let stream = self.get_mut();
        let mut inner = if let Some(inner) = stream.inner.get() {
            inner
        } else {
            return Poll::Ready(None);
        };

        match Pin::new(&mut inner).poll_read(ctx, &mut stream.buf) {
            Poll::Ready(Ok(read)) => {
                let data = stream.buf.drain(0..read).collect();
                stream.buf.resize(stream.cap, 0);

                let msg = (stream.map)(data);

                stream.inner.set(inner);
                Poll::Ready(Some(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Ready(Err(err)) => {
                let msg = (stream.map_err)(err);

                stream.inner.set(inner);
                Poll::Ready(Some(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Pending => {
                stream.inner.set(inner);
                Poll::Pending
            }
        }
    }
}

impl<A, W, M, N, T, E> raw::AsyncMessageFut for AsyncWriteFut<A, W, M, N, T, E>
where
    A: raw::Handler<T, Output = ()> + raw::Handler<E, Output = ()> + 'static,
    W: AsyncWrite + Unpin + Send,
    M: Fn((Vec<u8>, usize), Pin<Box<W>>) -> T + Unpin + Send + Sync,
    N: Fn(io::Error) -> E + Unpin + Send + Sync,
    T: Send + 'static,
    E: Send + 'static,
{
    type Actor = A;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut task::Context,
    ) -> Poll<raw::AsyncMessageRet<Self::Actor>> {
        let fut = self.get_mut();
        let mut inner = if let Some(inner) = fut.inner.get() {
            inner
        } else {
            return Poll::Ready(None);
        };

        match Pin::new(&mut inner).poll_write(ctx, fut.data.as_ref().unwrap()) {
            Poll::Ready(Ok(wrote)) => {
                let msg = (fut.map)((fut.data.take().unwrap(), wrote), inner);

                fut.inner.done();
                Poll::Ready(Some(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Ready(Err(err)) => {
                let msg = (fut.map_err)(err);

                fut.inner.done();
                Poll::Ready(Some(Box::new(AsyncMessage::new(msg))))
            }
            Poll::Pending => {
                fut.inner.set(inner);
                Poll::Pending
            }
        }
    }
}
