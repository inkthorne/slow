use crate::package::SlowPackage;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::net::UdpSocket;

/// A `SlowUdpSocket` represents a UDP connection that can send and receive `SlowPackage` packets.
///
/// This struct provides methods to create a new connection, send packages, receive packages, and retrieve the local address of the socket.
/// It is designed to work asynchronously using the Tokio runtime.
pub struct SlowUdpSocket {
    /// The UDP socket used for the connection.
    socket: UdpSocket,

    /// The count of packages sent.
    sent_packet_count: AtomicU32,

    /// The count of packages received.
    received_packet_count: AtomicU32,
}

impl SlowUdpSocket {
    /// Creates a new `SlowUdpSocket` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` that specifies the address to bind to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowUdpSocket` or an error if binding fails.
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(SlowUdpSocket {
            socket,
            sent_packet_count: AtomicU32::new(0),
            received_packet_count: AtomicU32::new(0),
        })
    }

    /// Sends a `SlowPackage` to the specified address.
    ///
    /// # Arguments
    ///
    /// * `package` - A reference to the `SlowPackage` to send.
    /// * `recipient_addr` - A reference to the `SocketAddr` of the recipient.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error if sending fails.
    pub async fn send_package(
        &self,
        package: &SlowPackage,
        recipient_addr: &SocketAddr,
    ) -> std::io::Result<()> {
        let packaged_data = package.package();
        self.send(&packaged_data, recipient_addr).await?;
        Ok(())
    }

    /// Sends raw data to the specified address.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of bytes to send.
    /// * `recipient_addr` - A reference to the `SocketAddr` of the recipient.
    ///
    /// # Returns
    ///
    /// * `Result<usize, std::io::Error>` - A result containing the number of bytes sent or an error if sending fails.
    pub async fn send(&self, data: &[u8], recipient_addr: &SocketAddr) -> std::io::Result<usize> {
        let bytes_sent = self.socket.send_to(data, *recipient_addr).await?;
        self.sent_packet_count.fetch_add(1, Ordering::SeqCst);
        Ok(bytes_sent)
    }

    /// Receives raw data from the socket.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable slice to write the received data into
    ///
    /// # Returns
    ///
    /// * `Option<(Vec<u8>, SocketAddr)>` - An option containing the received data as a Vec<u8> and the source address,
    ///   or `None` if an error occurs.
    pub async fn receive(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.received_packet_count.fetch_add(1, Ordering::SeqCst);
        self.socket.recv_from(buf).await
    }

    /// Receives a package from the socket.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowPackage, SocketAddr)>` - An option containing the received package and the source address, or `None` if an error occurs.
    pub async fn receive_package(&self) -> Option<(SlowPackage, SocketAddr)> {
        let mut buf = [0; 4096];
        if let Ok((size, src)) = self.receive(&mut buf).await {
            // Extract the package from the raw data
            // Note: No need to increment the counter since receive() already does that
            let stuff = buf[..size].to_vec();
            SlowPackage::unpackage(&stuff).map(|package| (package, src))
        } else {
            None
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

    /// Returns the count of packets sent.
    ///
    /// # Returns
    ///
    /// * `u32` - The count of packets sent.
    pub fn sent_packet_count(&self) -> u32 {
        self.sent_packet_count.load(Ordering::SeqCst)
    }

    /// Returns the count of packets received.
    ///
    /// # Returns
    ///
    /// * `u32` - The count of packets received.
    pub fn received_packet_count(&self) -> u32 {
        self.received_packet_count.load(Ordering::SeqCst)
    }
}
