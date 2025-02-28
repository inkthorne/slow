use crate::package::SlowPackage;
use crate::socket::SlowSocket;
use std::net::SocketAddr;

//=============================================================================
// SlowLinkPacketType
//=============================================================================

/// Represents the type of a SlowLink packet.
///
/// This enum defines the possible packet types that can be transmitted through a SlowLink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlowLinkPacketType {
    /// Indicates an acknowledgment packet.
    Acknowledge,
    /// Indicates a payload packet carrying data.
    Payload,
}

//=============================================================================
// SlowLinkPayloadPacket
//=============================================================================

/// A struct representing a payload packet sent through a SlowLink.
///
/// This struct uniquely identifies a payload packet with an ID.
pub struct SlowLinkPayloadPacket {
    /// The type of packet (see SlowLinkPacketType).
    pub packet_type: u8,
    /// The unique identifier for the packet.
    pub packet_id: u64,
}

impl SlowLinkPayloadPacket {
    /// Creates a new `SlowLinkPayloadPacket` with the packet type set to Payload.
    ///
    /// # Arguments
    ///
    /// * `packet_id` - The unique identifier for the packet
    ///
    /// # Returns
    ///
    /// * `SlowLinkPayloadPacket` - A new instance with packet_type set to Payload
    pub fn new(packet_id: u64) -> Self {
        Self {
            packet_type: SlowLinkPacketType::Payload as u8,
            packet_id,
        }
    }
}

//=============================================================================
// SlowLinkAckPacket
//=============================================================================

/// A struct representing an acknowledgment packet sent through a SlowLink.
///
/// This struct uniquely identifies an acknowledgment packet with an ID.
pub struct SlowLinkAckPacket {
    /// The type of packet (see SlowLinkPacketType).
    pub packet_type: u8,
    /// The highest unique packet identifier received by the sender.
    pub highest_packet_id: u64,
    /// A bitfield representing which packet ids have been received relative
    /// to the `higest_packet_id`.
    pub packet_bitfield: u64,
}

impl SlowLinkAckPacket {
    /// Creates a new `SlowLinkAckPacket` with the packet type set to Acknowledge.
    ///
    /// # Arguments
    ///
    /// * `packet_id` - The unique identifier for the packet
    ///
    /// # Returns
    ///
    /// * `SlowLinkAckPacket` - A new instance with packet_type set to Acknowledge
    pub fn new(highest_packet_id: u64, packet_bitfield: u64) -> Self {
        Self {
            packet_type: SlowLinkPacketType::Acknowledge as u8,
            highest_packet_id,
            packet_bitfield,
        }
    }
}

//=============================================================================
// SlowLink
//=============================================================================

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
        socket: &SlowSocket,
    ) -> std::io::Result<()> {
        // Send the package to the remote junction.
        socket.send_package(package, &self.remote_address).await?;
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
