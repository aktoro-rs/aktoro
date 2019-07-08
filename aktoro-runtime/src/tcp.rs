use std::future::Future;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task::Context as FutContext;
use std::task::Poll;

use aktoro_raw as raw;
use futures_core::Stream;
use futures_io::AsyncRead;
use futures_io::AsyncWrite;
use futures_io::Error as FutError;
use runtime::net;

use crate::error::Error;

/// A TCP client allowing to connect to
/// a TCP server and to communicate with it.
pub struct TcpClient {
    /// The TCP stream between the client
    /// and the server.
    stream: net::TcpStream,
}

/// A TCP server allowing to listen for
/// new TCP clients' connection and to
/// communicate with them.
pub struct TcpServer {
    /// The stream that receives new TCP
    /// connections.
    listener: net::TcpListener,
}

/// A future returned by
/// [`TcpClient::connect`] and that
/// resolves after the connection to the
/// TCP server has either failed or
/// succeeded.
///
/// [`TcpClient::connect`]: struct.TcpClient.html#method.connect
pub struct Connect {
    /// The actual future.
    connect: net::tcp::ConnectFuture,
}

/// A stream that yields new TCP
/// connections.
pub struct TcpIncoming<'i> {
    /// The actual stream.
    incoming: net::tcp::IncomingStream<'i>,
}

// TODO
pub struct OwnedTcpIcoming {
    // TODO
    server: TcpServer,
}

/// A TCP stream, owned by a server,
/// and allowing it to communicate with
/// a client.
pub struct TcpStream {
    /// The actual stream.
    stream: net::TcpStream,
}

impl raw::TcpClient for TcpClient {
    type Connect = Connect;

    type Error = Error;

    fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self::Connect, Error> {
        Ok(Connect {
            connect: net::TcpStream::connect(addr),
        })
    }
}

impl raw::TcpServer for TcpServer {
    type Stream = TcpStream;

    type Error = Error;

    fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, Error> {
        match net::TcpListener::bind(addr) {
            Ok(listener) => Ok(TcpServer { listener }),
            Err(err) => Err(Box::new(err).into()),
        }
    }

    fn local_addr(&self) -> Result<SocketAddr, Self::Error> {
        match self.listener.local_addr() {
            Ok(addr) => Ok(addr),
            Err(err) => Err(Box::new(err).into()),
        }
    }

    fn incoming<'i>(
        &'i mut self,
    ) -> Result<Box<dyn Stream<Item = Result<TcpStream, Error>> + Unpin + Send + 'i>, Error> {
        Ok(Box::new(TcpIncoming {
            incoming: self.listener.incoming(),
        }))
    }

    fn into_incoming(
        self,
    ) -> Result<Box<dyn Stream<Item = Result<TcpStream, Error>> + Unpin + Send>, Error> {
        Ok(Box::new(OwnedTcpIcoming { server: self }))
    }
}

impl raw::TcpStream for TcpClient {
    type Error = Error;

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        match self.stream.local_addr() {
            Ok(addr) => Ok(addr),
            Err(err) => Err(Box::new(err).into()),
        }
    }

    fn peer_addr(&self) -> Result<SocketAddr, Error> {
        match self.stream.peer_addr() {
            Ok(addr) => Ok(addr),
            Err(err) => Err(Box::new(err).into()),
        }
    }
}

impl raw::TcpStream for TcpStream {
    type Error = Error;

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        match self.stream.local_addr() {
            Ok(addr) => Ok(addr),
            Err(err) => Err(Box::new(err).into()),
        }
    }

    fn peer_addr(&self) -> Result<SocketAddr, Error> {
        match self.stream.peer_addr() {
            Ok(addr) => Ok(addr),
            Err(err) => Err(Box::new(err).into()),
        }
    }
}

impl Future for Connect {
    type Output = Result<TcpClient, Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Self::Output> {
        match Pin::new(&mut self.get_mut().connect).poll(ctx) {
            Poll::Ready(Ok(stream)) => Poll::Ready(Ok(TcpClient { stream })),
            Poll::Ready(Err(err)) => Poll::Ready(Err(Box::new(err).into())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'i> Stream for TcpIncoming<'i> {
    type Item = Result<TcpStream, Error>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.get_mut().incoming).poll_next(ctx) {
            Poll::Ready(Some(res)) => match res {
                Ok(stream) => Poll::Ready(Some(Ok(TcpStream { stream }))),
                Err(err) => Poll::Ready(Some(Err(Box::new(err).into()))),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Stream for OwnedTcpIcoming {
    type Item = Result<TcpStream, Error>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.get_mut().server.listener.accept()).poll(ctx) {
            Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(TcpStream { stream }))),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(Box::new(err).into()))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncRead for TcpClient {
    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
        buf: &mut [u8],
    ) -> Poll<Result<usize, FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_read(ctx, buf)
    }
}

impl AsyncWrite for TcpClient {
    fn poll_write(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
        buf: &[u8],
    ) -> Poll<Result<usize, FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_write(ctx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(ctx)
    }

    fn poll_close(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_close(ctx)
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
        buf: &mut [u8],
    ) -> Poll<Result<usize, FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_read(ctx, buf)
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        ctx: &mut FutContext,
        buf: &[u8],
    ) -> Poll<Result<usize, FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_write(ctx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(ctx)
    }

    fn poll_close(self: Pin<&mut Self>, ctx: &mut FutContext) -> Poll<Result<(), FutError>> {
        Pin::new(&mut self.get_mut().stream).poll_close(ctx)
    }
}
