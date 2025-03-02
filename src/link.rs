use crate::link_packet::{SlowLinkPacketType, SlowLinkPayloadPacket};
use crate::tracker::PacketTracker;
use crate::{package::SlowPackage, tracker::UpdateResult};
use bincode;
use std::net::SocketAddr;

//=============================================================================
// UnpackResult
//=============================================================================
/// Represents the result of unpacking a packet.
///
/// This enum defines the possible outcomes of unpacking a packet.
#[derive(Debug, PartialEq)]
pub enum UnpackResult {
    /// Indicates that the packet was successfully unpacked and provides the starting index of payload data.
    Payload(usize),
    /// Indicates that the packet was a control packet.
    Control,
    /// Indicates that the packet was a duplicate packet and was discarded.
    Duplicate,
    /// Indicates that the packet was too old to be tracked and was discarded.
    Old,
    /// Indicates that the packet was invalid or malformed.
    Invalid,
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
    /// Counter for packets successfully sent through this link.
    packed_count: u64,
    /// Packet state tracking for received packets.
    unpacked_tracker: PacketTracker,
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
            packed_count: 0,
            unpacked_tracker: PacketTracker::new(),
        })
    }

    /// Creates a packet buffer from a `SlowPackage`.
    ///
    /// This method puts the package into a buffer preceded by a SlowLinkPayloadPacket header.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` to pack
    ///
    /// # Returns
    ///
    /// * `Option<Vec<u8>>` - The packed buffer containing header and package data, or None if serialization fails
    pub fn pack(&mut self, package: &SlowPackage) -> Option<Vec<u8>> {
        // Create a new payload packet with the current packets_sent as the ID
        // Increment the packages_sent counter
        let packed_count = self.packed_count + 1;
        let payload_packet = SlowLinkPayloadPacket::new(packed_count);

        // Serialize the payload packet into a buffer
        let payload_header = match bincode::serialize(&payload_packet) {
            Ok(header) => header,
            Err(_) => return None,
        };

        // Get the package data
        let package_data = package.package();

        // Combine the payload header and package data
        let mut buffer = Vec::with_capacity(payload_header.len() + package_data.len());
        buffer.extend_from_slice(&payload_header);
        buffer.extend_from_slice(&package_data);

        // Increment the packages_sent counter
        self.packed_count = packed_count;
        Some(buffer)
    }

    /// Unpacks a received packet and determine its type.
    ///
    /// This method analyzes the provided byte slice to determine whether it contains
    /// a payload packet or an acknowledgment packet. For payload packets, it returns
    /// the index where payload data begins (after the SlowLinkPayloadPacket header).
    /// For acknowledgment packets, it returns None.
    ///
    /// # Arguments
    ///
    /// * `data` - The received packet as a byte slice
    ///
    /// # Returns
    ///
    /// * `UnpackResult` - The result of unpacking the packet
    pub fn unpack(&mut self, data: &[u8]) -> UnpackResult {
        // Check if data is empty
        if data.is_empty() {
            return UnpackResult::Control;
        }

        // Match on the packet type
        match SlowLinkPacketType::try_from(data[0]) {
            Ok(SlowLinkPacketType::Payload) => self.process_payload(data),
            Ok(SlowLinkPacketType::Acknowledge) => Self::process_ack(data),
            Ok(SlowLinkPacketType::Hello) => UnpackResult::Control,
            Err(_) => UnpackResult::Invalid,
        }
    }

    /// Process a payload packet and extract the starting index of its data.
    ///
    /// # Arguments
    ///
    /// * `data` - The received payload packet as a byte slice
    ///
    /// # Returns
    ///
    /// * `UnpackResult` - The result of processing the payload packet
    fn process_payload(&mut self, data: &[u8]) -> UnpackResult {
        // Try to deserialize as a payload packet
        if let Ok(payload_packet) = bincode::deserialize::<SlowLinkPayloadPacket>(
            &data[0..std::mem::size_of::<SlowLinkPayloadPacket>()],
        ) {
            // Update the packet state with this new packet ID
            if self.unpacked_tracker.update(payload_packet.packet_id) != UpdateResult::Success {
                return UnpackResult::Duplicate;
            }

            // Return the index where payload data starts
            let payload_start = std::mem::size_of::<SlowLinkPayloadPacket>();
            return UnpackResult::Payload(payload_start);
        }
        UnpackResult::Control
    }

    /// Process an acknowledgment packet.
    ///
    /// # Arguments
    ///
    /// * `data` - The received acknowledgment packet as a byte slice
    ///
    /// # Returns
    ///
    /// * `UnpackResult` - Always UnpackResult::Control for acknowledgment packets
    fn process_ack(_data: &[u8]) -> UnpackResult {
        UnpackResult::Control
    }

    /// Returns the remote junction address.
    ///
    /// # Returns
    ///
    /// * `SocketAddr` - The remote junction address
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }

    /// Returns the count of packets successfully sent through this link.
    ///
    /// # Returns
    ///
    /// * `u64` - The count of successfully sent packets.
    pub fn packed_count(&self) -> u64 {
        self.packed_count
    }
}
