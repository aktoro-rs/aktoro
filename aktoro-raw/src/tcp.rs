use std::error::Error as StdError;
use std::future::Future;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use futures_core::Stream;
use futures_io::AsyncRead;
use futures_io::AsyncWrite;

pub type TcpServerIncoming<'s, S, E> = Box<dyn Stream<Item = Result<S, E>> + 's>;

pub trait TcpClient: TcpStream + Sized {
    type Connect: Future<Output = Result<Self, <Self as TcpClient>::Error>>;

    type Error: StdError;

    /// Tries to connect to a TCP server at the
    /// given address.
    fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self::Connect, <Self as TcpClient>::Error>;
}

pub trait TcpServer: Sized {
    type Stream: TcpStream;

    type Error: StdError;

    /// Tries to create a new TCP server that
    /// will be bound to the given address.
    fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, Self::Error>;

    /// Returns the address that the server
    /// it bound to.
    fn local_addr(&self) -> Result<SocketAddr, Self::Error>;

    /// Returns a stream of incoming connections.
    fn incoming(&mut self) -> Result<TcpServerIncoming<Self::Stream, Self::Error>, Self::Error>;
}

pub trait TcpStream: AsyncRead + AsyncWrite {
    type Error: StdError;

    /// Returns the address that the server
    /// is bound to.
    fn local_addr(&self) -> Result<SocketAddr, Self::Error>;

    /// Returns the address of the client
    /// this stream is connected to.
    fn peer_addr(&self) -> Result<SocketAddr, Self::Error>;
}
