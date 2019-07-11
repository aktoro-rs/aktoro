use std::error::Error as StdError;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use futures_core::Future;

pub type UdpSocketSendTo<'s, E> = Box<dyn Future<Output = Result<usize, E>> + 's>;

pub type UdpSocketRecv<'s, E> = Box<dyn Future<Output = Result<(usize, SocketAddr), E>> + 's>;

pub trait UdpSocket: Unpin + Send + Sized {
    type Error: StdError + Send + 'static;

    /// Tries to create a new UDP socket that
    /// will be bound to the given address.
    fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, Self::Error>;

    /// Returns the address that the socket
    /// is bound to.
    fn local_addr(&self) -> Result<SocketAddr, Self::Error>;

    /// Tries to send data to the given
    /// address, eventually returning a future
    /// that will resolve with the number of
    /// bytes sent.
    fn send_to<'s, A: ToSocketAddrs>(
        &'s mut self,
        buf: &'s [u8],
        addr: A,
    ) -> Result<UdpSocketSendTo<'s, Self::Error>, Self::Error>;

    /// Tries to receive data and to write it
    /// to the buffer, eventually returning a
    /// future that will resolve with the
    /// number of bytes received and the
    /// address of the data's sender.
    fn recv<'s>(
        &'s mut self,
        buf: &'s mut [u8],
    ) -> Result<UdpSocketRecv<'s, Self::Error>, Self::Error>;
}
