use crate::package::SlowPackage;
use crate::socket::SlowSocket;
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
// SlowLinkPacketTracker
//=============================================================================
/// Tracks packet receipt information in SlowLink connections.
///
/// This struct monitors the highest packet ID received and maintains a bitfield
/// to efficiently track recently received packets, enabling duplicate detection
/// and reliable packet delivery confirmation.
pub struct SlowLinkPacketTracker {
    /// The ID of the highest packet received.
    highest_packet_id: u64,

    /// A bitfield used to track recently received packets relative to highest_packet_id.
    packet_bitfield: u64,
}

impl SlowLinkPacketTracker {
    /// Creates a new `SlowLinkPacketTracker`.
    ///
    /// # Returns
    ///
    /// A new instance of `SlowLinkPacketTracker` with default values.
    pub fn new() -> Self {
        SlowLinkPacketTracker {
            highest_packet_id: 0,
            packet_bitfield: 0,
        }
    }

    /// Updates the packet information with a new packet ID.
    ///
    /// # Arguments
    ///
    /// * `packet_id` - The ID of the packet to update.
    ///
    /// # Returns
    ///
    /// `true` if the packet information was updated successfully, `false` otherwise.
    pub fn update(&mut self, packet_id: u64) -> bool {
        let shift = packet_id as i64 - self.highest_packet_id as i64;

        // If the packet is the same ID we already received or it's too old, ignore it
        if shift == 0 || shift < -63 {
            return false;
        }

        // If the packet is older than the highest packet ID but within the bitfield range
        if shift < 0 {
            let mask = 1 << -shift;

            // Check if we've already received this packet
            if self.packet_bitfield & mask != 0 {
                return false;
            }

            // Mark this packet as received
            self.packet_bitfield |= mask;
            return true;
        }

        // If the packet is newer than the highest packet ID
        if shift > 63 {
            // Reset the bitfield and set the lowest bit
            self.packet_bitfield = 0;
            self.packet_bitfield |= 1;
        } else {
            // Shift the bitfield and set the lowest bit
            self.packet_bitfield <<= shift;
            self.packet_bitfield |= 1;
        }

        // Update the highest packet ID
        self.highest_packet_id = packet_id;

        true
    }

    /// Gets the highest packet ID received.
    ///
    /// # Returns
    ///
    /// The highest packet ID received.
    pub fn highest_packet_id(&self) -> u64 {
        self.highest_packet_id
    }

    /// Gets the packet bitfield.
    ///
    /// # Returns
    ///
    /// The packet bitfield.
    pub fn packet_bitfield(&self) -> u64 {
        self.packet_bitfield
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
    /// Counter for packets successfully sent through this link.
    packets_sent: u64,
    /// Packet state tracking for received packets.
    packet_state: SlowLinkPacketTracker,
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
            packets_sent: 0,
            packet_state: SlowLinkPacketTracker::new(),
        })
    }

    /// Sends a `SlowPackage` to the remote junction.
    ///
    /// This method puts the package into a buffer preceded by a SlowLinkPayloadPacket
    /// and sends that buffer using the socket.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` to send
    /// * `socket` - A `SlowSocket` that can send data
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error
    pub async fn send(
        &mut self,
        package: &SlowPackage,
        socket: &SlowSocket,
    ) -> std::io::Result<()> {
        // Create a new payload packet with the current packets_sent as the ID
        let payload_packet = SlowLinkPayloadPacket::new(self.packets_sent);

        // Serialize the payload packet into a buffer
        let payload_header = bincode::serialize(&payload_packet)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Get the package data
        let package_data = package.package();

        // Combine the payload header and package data
        let mut buffer = Vec::with_capacity(payload_header.len() + package_data.len());
        buffer.extend_from_slice(&payload_header);
        buffer.extend_from_slice(&package_data);

        // Send the combined buffer to the remote junction
        socket.send(&buffer, &self.remote_address).await?;

        // Increment the packages_sent counter on success
        self.packets_sent += 1;

        Ok(())
    }

    /// Process a received packet and determine its type.
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
    /// * `Option<usize>` - Some containing the starting index of payload data for payload packets, or None for ack packets
    pub fn process(&mut self, data: &[u8]) -> Option<usize> {
        // Check if data is empty
        if data.is_empty() {
            return None;
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
    /// * `Option<usize>` - Some containing the starting index of payload data if valid, or None if invalid
    fn process_payload(&mut self, data: &[u8]) -> Option<usize> {
        // Try to deserialize as a payload packet
        if let Ok(payload_packet) = bincode::deserialize::<SlowLinkPayloadPacket>(
            &data[0..std::mem::size_of::<SlowLinkPayloadPacket>()],
        ) {
            // Update the packet state with this new packet ID
            self.packet_state.update(payload_packet.packet_id);

            // Return the index where payload data starts
            let payload_start = std::mem::size_of::<SlowLinkPayloadPacket>();
            Some(payload_start)
        } else {
            None
        }
    }

    /// Process an acknowledgment packet.
    ///
    /// # Arguments
    ///
    /// * `data` - The received acknowledgment packet as a byte slice
    ///
    /// # Returns
    ///
    /// * `Option<usize>` - Always None for acknowledgment packets
    fn process_ack(_data: &[u8]) -> Option<usize> {
        None
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
    pub fn packets_sent(&self) -> u64 {
        self.packets_sent
    }
}
