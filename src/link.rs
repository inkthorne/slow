use crate::tracker::PacketTracker;
use crate::{package::SlowPackage, tracker::UpdateResult};
use bincode; // Add bincode for serialization
use serde::{Deserialize, Serialize};
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

impl From<u8> for SlowLinkPacketType {
    /// Converts a `u8` value into a `SlowLinkPacketType`.
    ///
    /// This function maps specific `u8` values to their corresponding `SlowLinkPacketType` variants.
    /// If the provided value doesn't match any known packet type, it defaults to `SlowLinkPacketType::Payload`.
    ///
    /// # Arguments
    ///
    /// * `value` - The `u8` value to convert.
    ///
    /// # Returns
    ///
    /// * `SlowLinkPacketType` - The corresponding enum variant.
    fn from(value: u8) -> Self {
        match value {
            0 => SlowLinkPacketType::Acknowledge,
            1 => SlowLinkPacketType::Payload,
            _ => SlowLinkPacketType::Payload, // Default to Payload for unknown values
        }
    }
}

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
}

//=============================================================================
// SlowLinkPayloadPacket
//=============================================================================
/// A struct representing a payload packet sent through a SlowLink.
///
/// This struct uniquely identifies a payload packet with an ID.
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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
// SlowLinkPacket
//=============================================================================
/// Represents the result of processing a packet.
///
/// This enum contains either a payload packet with its data or an acknowledgment packet.
#[derive(Debug)]
pub enum SlowLinkPacket {
    /// A payload packet with its ID and the associated data
    Payload {
        /// The unique identifier for the packet
        packet_id: u64,
        /// The payload data
        data: Vec<u8>,
    },
    /// An acknowledgment packet
    Acknowledge {
        /// The highest unique packet identifier received
        highest_packet_id: u64,
        /// A bitfield representing which packets have been received
        packet_bitfield: u64,
    },
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
        let packet_type = SlowLinkPacketType::from(data[0]);
        match packet_type {
            SlowLinkPacketType::Payload => self.process_payload(data),
            SlowLinkPacketType::Acknowledge => Self::process_ack(data),
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
