use std::future::Future;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::pin::Pin;
use std::task;
use std::task::Poll;

use aktoro_raw as raw;
use runtime::net;

use crate::error::Error;

/// A UDP socket, allowing to listen for
/// new connections and to connect and
/// communicate with other sockets.
pub struct UdpSocket {
    /// The actual socket.
    socket: net::UdpSocket,
}

/// A future returned by [`UdpSocket::send_to`]
/// and that resolves after sending the data
/// has either failed or succeeded, in which
/// case it will also return the number of
/// bytes sent.
///
/// [`UdpSocket::send_to`]: struct.UdpSocket.html#method.send_to
pub struct SendTo<'s, 'b> {
    /// The actual future.
    send_to: net::udp::SendToFuture<'s, 'b>,
}

/// A future returned by [`UdpSocket::recv`]
/// and that resolves after trying to
/// receive data has either failed or
/// succeeded, in which case it will also
/// return the number of bytes received and
/// the address of the sender.
///
/// [`UdpSocket::recv`]: struct.UdpSocket.html#method.recv
pub struct Recv<'s, 'b> {
    /// The actual future.
    recv_from: net::udp::RecvFromFuture<'s, 'b>,
}

impl raw::UdpSocket for UdpSocket {
    type Error = Error;

    fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, Error> {
        match net::UdpSocket::bind(addr) {
            Ok(socket) => Ok(UdpSocket { socket }),
            Err(err) => Err(Box::new(err).into()),
        }
    }

    fn local_addr(&self) -> Result<SocketAddr, Error> {
        match self.socket.local_addr() {
            Ok(addr) => Ok(addr),
            Err(err) => Err(Box::new(err).into()),
        }
    }

    fn send_to<'s, A: ToSocketAddrs>(
        &'s mut self,
        buf: &'s [u8],
        addr: A,
    ) -> Result<raw::UdpSocketSendTo<'s, Error>, Error> {
        Ok(Box::new(SendTo {
            send_to: self.socket.send_to(buf, addr),
        }))
    }

    fn recv<'s>(&'s mut self, buf: &'s mut [u8]) -> Result<raw::UdpSocketRecv<'s, Error>, Error> {
        Ok(Box::new(Recv {
            recv_from: self.socket.recv_from(buf),
        }))
    }
}

impl<'s, 'b> Future for SendTo<'s, 'b> {
    type Output = Result<usize, Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.get_mut().send_to).poll(ctx) {
            Poll::Ready(Ok(sent)) => Poll::Ready(Ok(sent)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(Box::new(err).into())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'s, 'b> Future for Recv<'s, 'b> {
    type Output = Result<(usize, SocketAddr), Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut task::Context) -> Poll<Self::Output> {
        match Pin::new(&mut self.get_mut().recv_from).poll(ctx) {
            Poll::Ready(Ok(recved)) => Poll::Ready(Ok(recved)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(Box::new(err).into())),
            Poll::Pending => Poll::Pending,
        }
    }
}
