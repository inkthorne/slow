use crate::datagram::SlowDatagram;
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Clone, PartialEq, Debug)]
pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

/// A `SlowConnection` represents a UDP connection that can send and receive `SlowDatagram` packets.
///
/// This struct provides methods to create a new connection, send datagrams, receive datagrams, and retrieve the local address of the socket.
/// It is designed to work asynchronously using the Tokio runtime.
pub struct SlowConnection {
    socket: UdpSocket,
}

impl SlowConnection {
    /// Creates a new `SlowConnection` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` that specifies the address to bind to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowConnection` or an error if binding fails.
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(SlowConnection { socket })
    }

    /// Sends a `SlowDatagram` to the specified address.
    ///
    /// # Arguments
    ///
    /// * `datagram` - A reference to the `SlowDatagram` to send.
    /// * `recipient_addr` - A reference to the `SocketAddr` of the recipient.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error if sending fails.
    pub async fn send_datagram(&self, datagram: &SlowDatagram, recipient_addr: &SocketAddr) -> std::io::Result<()> {
        let packaged_data = datagram.package();
        self.socket.send_to(&packaged_data, *recipient_addr).await?;
        Ok(())
    }

    /// Receives a datagram from the socket.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowDatagram, SocketAddr)>` - An option containing the received datagram and the source address, or `None` if an error occurs.
    pub async fn recv_datagram(&self) -> Option<(SlowDatagram, SocketAddr)> {
        let mut buf = [0; 4096];
        match self.socket.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                let datagram = &buf[..amt];
                SlowDatagram::unpackage(datagram).map(|d| (d, src))
            }
            Err(_) => None,
        }
    }

    /// Returns the local address of the socket.
    ///
    /// # Returns
    ///
    /// * `Result<SocketAddr, std::io::Error>` - A result containing the local address or an error if unable to retrieve it.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}
