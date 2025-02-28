use crate::connection::SlowSocket;
use crate::package::SlowPackage;
use std::net::SocketAddr;

/// A `SlowLink` represents a direct connection between two junctions in the network.
///
/// This struct provides methods to create a new link and send packages through it.
/// It is designed to simplify direct communication between two specific junctions.
pub struct SlowLink {
    /// The remote junction address.
    remote_address: SocketAddr,
    /// Counter for packages successfully sent through this link.
    packages_sent: u64,
}

impl SlowLink {
    /// Creates a new `SlowLink` instance.
    ///
    /// # Arguments
    ///
    /// * `remote_addr` - The address of the remote junction
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowLink` or an error
    pub fn new(remote_address: SocketAddr) -> std::io::Result<Self> {
        Ok(Self {
            remote_address,
            packages_sent: 0,
        })
    }

    /// Sends a `SlowPackage` to the remote junction.
    ///
    /// Note: This method requires a connection implementation that should be provided by the caller.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` to send
    /// * `connection` - A connection object that can send packages
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error
    pub async fn send(
        &mut self,
        package: &SlowPackage,
        connection: &SlowSocket,
    ) -> std::io::Result<()> {
        // Send the package to the remote junction.
        connection
            .send_package(package, &self.remote_address)
            .await?;
        // Increment the packages_sent counter on success.
        self.packages_sent += 1;
        Ok(())
    }

    /// Returns the remote junction address.
    ///
    /// # Returns
    ///
    /// * `SocketAddr` - The remote junction address
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }

    /// Returns the count of packages successfully sent through this link.
    ///
    /// # Returns
    ///
    /// * `u64` - The count of successfully sent packages
    pub fn packages_sent(&self) -> u64 {
        self.packages_sent
    }
}
