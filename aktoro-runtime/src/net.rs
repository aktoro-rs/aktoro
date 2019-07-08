use aktoro_raw as raw;

use crate::tcp::TcpClient;
use crate::tcp::TcpServer;
use crate::udp::UdpSocket;

pub struct NetworkManager;

impl raw::NetworkManager for NetworkManager {
    type TcpClient = TcpClient;
    type TcpServer = TcpServer;

    type UdpSocket = UdpSocket;
}
