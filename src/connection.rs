use crate::package::SlowPackage;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::net::UdpSocket;

/// A `SlowConnection` represents a UDP connection that can send and receive `SlowPackage` packets.
///
/// This struct provides methods to create a new connection, send packages, receive packages, and retrieve the local address of the socket.
/// It is designed to work asynchronously using the Tokio runtime.
pub struct SlowConnection {
    /// The UDP socket used for the connection.
    socket: UdpSocket,

    /// The count of packages sent.
    sent_package_count: AtomicU32,

    /// The count of packages received.
    received_package_count: AtomicU32,
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
        Ok(SlowConnection {
            socket,
            sent_package_count: AtomicU32::new(0),
            received_package_count: AtomicU32::new(0),
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
        self.socket.send_to(&packaged_data, *recipient_addr).await?;
        self.sent_package_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Receives a package from the socket.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowPackage, SocketAddr)>` - An option containing the received package and the source address, or `None` if an error occurs.
    pub async fn receive_package(&self) -> Option<(SlowPackage, SocketAddr)> {
        let mut buf = [0; 4096];
        match self.socket.recv_from(&mut buf).await {
            Ok((amt, src)) => {
                let package = &buf[..amt];
                SlowPackage::unpackage(package).map(|d| {
                    self.received_package_count.fetch_add(1, Ordering::SeqCst);
                    (d, src)
                })
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

    /// Returns the count of packages sent.
    ///
    /// # Returns
    ///
    /// * `u32` - The count of packages sent.
    pub fn sent_package_count(&self) -> u32 {
        self.sent_package_count.load(Ordering::SeqCst)
    }

    /// Returns the count of packages received.
    ///
    /// # Returns
    ///
    /// * `u32` - The count of packages received.
    pub fn received_package_count(&self) -> u32 {
        self.received_package_count.load(Ordering::SeqCst)
    }
}
