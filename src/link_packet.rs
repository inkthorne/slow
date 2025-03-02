use serde::{Deserialize, Serialize};

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
    /// Indicates a hello packet.
    Hello,
    /// Indicates a payload packet carrying data.
    Payload,
}

impl TryFrom<u8> for SlowLinkPacketType {
    type Error = &'static str;

    /// Attempts to convert a `u8` value into a `SlowLinkPacketType`.
    ///
    /// This function maps specific `u8` values to their corresponding `SlowLinkPacketType` variants.
    /// Returns an error if the value doesn't match any known packet type.
    ///
    /// # Arguments
    ///
    /// * `value` - The `u8` value to convert.
    ///
    /// # Returns
    ///
    /// * `Result<SlowLinkPacketType, &'static str>` - The corresponding enum variant or an error.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SlowLinkPacketType::Acknowledge),
            1 => Ok(SlowLinkPacketType::Hello),
            2 => Ok(SlowLinkPacketType::Payload),
            _ => Err("Invalid packet type"),
        }
    }
}

//=============================================================================
// SlowLinkPacketHeader
//=============================================================================
/// A struct representing a payload packet sent through a SlowLink.
///
/// This struct uniquely identifies a payload packet with an ID.
#[derive(Serialize, Deserialize)]
pub struct SlowLinkPacketHeader {
    /// The type of packet (see SlowLinkPacketType).
    pub packet_type: u8,
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
