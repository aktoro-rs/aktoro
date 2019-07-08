use std::net::ToSocketAddrs;

use crate::tcp::TcpClient;
use crate::tcp::TcpServer;
use crate::udp::UdpSocket;

type TcpConnectRes<C> = Result<<C as TcpClient>::Connect, <C as TcpClient>::Error>;

type TcpBindRes<S> = Result<S, <S as TcpServer>::Error>;

type UdpBindRes<S> = Result<S, <S as UdpSocket>::Error>;

pub trait NetworkManager: Unpin + Send + Sync {
    /// The type of the TCP socket client
    /// that actors can use to be compatible
    /// with the runtime (this might not be
    /// necessary depending on the runtime
    /// implementation).
    type TcpClient: TcpClient;

    /// The type of the TCP socket server
    /// that actors can use to be compatible
    /// with the runtime (this might not be
    /// necessary depending on the runtime
    /// implementation).
    type TcpServer: TcpServer;

    /// The type of UDP socket that actors
    /// can use to be compatible with the
    /// runtime (this might not be necessary
    /// depending on the runtime implementation).
    type UdpSocket: UdpSocket;

    /// Tries to connect to a TCP server at the
    /// given address.
    fn tcp_connect<A: ToSocketAddrs>(&self, addr: A) -> TcpConnectRes<Self::TcpClient> {
        Self::TcpClient::connect(addr)
    }

    /// Tries to create a new TCP server that
    /// will be bound to the given address.
    fn tcp_bind<A: ToSocketAddrs>(&self, addr: A) -> TcpBindRes<Self::TcpServer> {
        Self::TcpServer::bind(addr)
    }

    /// Tries to create a new UDP socket that
    /// will be bound to the given address.
    fn udp_bind<A: ToSocketAddrs>(&self, addr: A) -> UdpBindRes<Self::UdpSocket> {
        Self::UdpSocket::bind(addr)
    }
}
