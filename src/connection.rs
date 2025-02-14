use crate::datagram::SlowDatagram;
use serde_json::Value;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Clone, PartialEq, Debug)]
pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

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
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowConnection` or an error.
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(SlowConnection { socket })
    }

    /// Sends a `SlowDatagram` to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - A reference to the `SocketAddr` of the recipient.
    /// * `datagram` - A reference to the `SlowDatagram` to send.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error.
    pub async fn send_datagram(&self, addr: &SocketAddr, datagram: &SlowDatagram) -> std::io::Result<()> {
        let packaged_data = datagram.package();
        self.socket.send_to(&packaged_data, *addr).await?;
        Ok(())
    }

    /// Receives a datagram from the socket.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowDatagram, SocketAddr)>` - An option containing the received datagram and the source address or `None` if no datagram is available.
    pub async fn recv(&self) -> Option<(SlowDatagram, SocketAddr)> {
        let mut buf = [0; 1024];
        match self.socket.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                let datagram = &buf[..amt];
                SlowDatagram::unpackage(datagram).map(|d| (d, src))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }

    /// Waits for a datagram to be received and unpackages it into a `SlowDatagram`.
    ///
    /// # Returns
    ///
    /// * `Result<(SlowDatagram, SocketAddr), std::io::Error>` - A result containing the received datagram and the source address or an error.
    pub async fn wait_for_datagram(&self) -> std::io::Result<(SlowDatagram, SocketAddr)> {
        let mut buf = [0; 1024];
        let (amt, src) = self.socket.recv_from(&mut buf).await?;
        let datagram = &buf[..amt];
        SlowDatagram::unpackage(datagram).map(|d| (d, src)).ok_or(std::io::Error::new(ErrorKind::InvalidData, "Failed to unpackage datagram"))
    }

    /// Returns the local address of the socket.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}
