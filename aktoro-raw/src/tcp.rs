use std::error;
use std::future::Future;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use futures_core::Stream;
use futures_io::AsyncRead;
use futures_io::AsyncWrite;

pub type TcpServerIncoming<'s, S> = Box<
    dyn Stream<Item = Result<<S as TcpServer>::Stream, <S as TcpServer>::Error>>
        + Unpin
        + Send
        + 's,
>;
pub type OwnedTcpServerIncoming<S> = Box<
    dyn Stream<Item = Result<<S as TcpServer>::Stream, <S as TcpServer>::Error>> + Unpin + Send,
>;

pub trait TcpClient: TcpStream + Unpin + Send + Sized {
    type Connect: Future<Output = Result<Self, <Self as TcpClient>::Error>> + Unpin + Send;

    type Error: error::Error + Send + 'static;

    /// Tries to connect to a TCP server at the
    /// given address.
    fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self::Connect, <Self as TcpClient>::Error>;
}

pub trait TcpServer: Unpin + Send + Sized {
    type Stream: TcpStream;

    type Error: error::Error + Send + 'static;

    /// Tries to create a new TCP server that
    /// will be bound to the given address.
    fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, Self::Error>;

    /// Returns the address that the server
    /// it bound to.
    fn local_addr(&self) -> Result<SocketAddr, Self::Error>;

    /// Returns a stream of incoming connections.
    fn incoming(&mut self) -> Result<TcpServerIncoming<Self>, Self::Error>;

    /// Returns a stream of incoming connections,
    /// consuming the server.
    fn into_incoming(self) -> Result<OwnedTcpServerIncoming<Self>, Self::Error>;
}

pub trait TcpStream: AsyncRead + AsyncWrite + Unpin + Send {
    type Error: error::Error + Send + 'static;

    /// Returns the address that the server
    /// is bound to.
    fn local_addr(&self) -> Result<SocketAddr, Self::Error>;

    /// Returns the address of the client
    /// this stream is connected to.
    fn peer_addr(&self) -> Result<SocketAddr, Self::Error>;
}
