use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};

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
            _ => Err("Invalid packet type."),
        }
    }
}

//=============================================================================
// SlowLinkPayloadPacket
//=============================================================================
/// A struct representing a payload packet sent through a SlowLink.
///
/// This struct uniquely identifies a payload packet with an ID.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SlowLinkPayloadPacket {
    /// The type of packet (see SlowLinkPacketType).
    pub packet_type: u8,
    /// The unique identifier for the packet.
    pub packet_id: u64,
    /// The size of the payload data in bytes.
    pub payload_size: u16,
    /// The payload data carried by the packet.
    pub payload: Vec<u8>,
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
    pub fn new(packet_id: u64, payload: Vec<u8>) -> Self {
        Self {
            packet_type: SlowLinkPacketType::Payload as u8,
            packet_id,
            payload_size: payload.len() as u16,
            payload,
        }
    }

    /// Packs the packet data into a byte vector.
    ///
    /// This method serializes the packet's fields into a Vec<u8> using the byteorder crate
    /// with a Cursor writer. The packet_type is written as a u8, followed by the packet_id as a u64
    /// in big-endian format, the payload_size as a u16 in big-endian format, and finally the payload data.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The serialized packet as a byte vector
    pub fn pack(&self) -> Vec<u8> {
        // Calculate the total size needed for the buffer:
        // 1 byte for packet_type + 8 bytes for packet_id + 2 bytes for payload_size + payload length
        let total_size = 1 + 8 + 2 + self.payload.len();
        let mut buffer = Vec::with_capacity(total_size);

        // Write packet_type as u8
        buffer.write_u8(self.packet_type).unwrap();

        // Write packet_id as u64 in big-endian format
        buffer.write_u64::<BigEndian>(self.packet_id).unwrap();

        // Write payload_size as u16 in big-endian format
        buffer.write_u16::<BigEndian>(self.payload_size).unwrap();

        // Write the payload data by extending the buffer
        buffer.extend_from_slice(&self.payload);

        buffer
    }

    /// Unpacks a byte slice into a `SlowLinkPayloadPacket`.
    ///
    /// This method deserializes a byte slice into a `SlowLinkPayloadPacket` instance.
    /// It reads the packet_type, packet_id, payload_size, and the payload data.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the serialized packet data
    ///
    /// # Returns
    ///
    /// * `Result<Self, &'static str>` - A Result containing either the deserialized packet or an error
    pub fn unpack(data: &[u8]) -> Result<Self, &'static str> {
        // Check if the data has at least the header size (1 + 8 + 2 bytes)
        const HEADER_SIZE: usize = 1 + 8 + 2;
        if data.len() < HEADER_SIZE {
            return Err("Data is too short to contain a valid packet header");
        }

        let mut cursor = Cursor::new(data);

        // Read packet_type
        let packet_type = match cursor.read_u8() {
            Ok(pt) => pt,
            Err(_) => return Err("Failed to read packet type"),
        };

        // Validate packet type
        match SlowLinkPacketType::try_from(packet_type) {
            Ok(SlowLinkPacketType::Payload) => {}
            Ok(_) => return Err("Not a payload packet"),
            Err(e) => return Err(e),
        }

        // Read packet_id
        let packet_id = match cursor.read_u64::<BigEndian>() {
            Ok(id) => id,
            Err(_) => return Err("Failed to read packet id"),
        };

        // Read payload_size
        let payload_size = match cursor.read_u16::<BigEndian>() {
            Ok(size) => size,
            Err(_) => return Err("Failed to read payload size"),
        };

        // Check if the data has enough bytes for the payload
        let position = cursor.position() as usize;
        if data.len() - position < payload_size as usize {
            return Err("Data is too short to contain the specified payload");
        }

        // Read payload data
        let mut payload = vec![0; payload_size as usize];
        if let Err(_) = cursor.read_exact(&mut payload) {
            return Err("Failed to read payload data");
        }

        Ok(Self {
            packet_type,
            packet_id,
            payload_size,
            payload,
        })
    }
}

//=============================================================================
// SlowLinkAckPacket
//=============================================================================
/// A struct representing an acknowledgment packet sent through a SlowLink.
///
/// This struct uniquely identifies an acknowledgment packet with an ID.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SlowLinkAckPacket {
    /// The type of packet (see SlowLinkPacketType).
    pub packet_type: u8,
    /// The highest unique packet identifier received by the sender.
    pub highest_packet_id: u64,
    /// A bitfield representing which packet ids have been received relative
    /// to the `highest_packet_id`.
    pub packet_bitfield: u64,
}

impl SlowLinkAckPacket {
    /// Creates a new `SlowLinkAckPacket` with the packet type set to Acknowledge.
    ///
    /// # Arguments
    ///
    /// * `highest_packet_id` - The highest packet ID received so far
    /// * `packet_bitfield` - Bitfield representing received packets relative to highest_packet_id
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

    /// Packs the packet data into a byte vector.
    ///
    /// This method serializes the packet's fields into a Vec<u8> using the byteorder crate
    /// with a Cursor writer. The packet_type is written as a u8, followed by the highest_packet_id
    /// and packet_bitfield as u64 values in big-endian format.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The serialized packet as a byte vector
    pub fn pack(&self) -> Vec<u8> {
        // Calculate the total size needed for the buffer:
        // 1 byte for packet_type + 8 bytes for highest_packet_id + 8 bytes for packet_bitfield
        let total_size = 1 + 8 + 8;
        let mut buffer = Vec::with_capacity(total_size);

        // Write packet_type as u8
        buffer.write_u8(self.packet_type).unwrap();

        // Write highest_packet_id as u64 in big-endian format
        buffer
            .write_u64::<BigEndian>(self.highest_packet_id)
            .unwrap();

        // Write packet_bitfield as u64 in big-endian format
        buffer.write_u64::<BigEndian>(self.packet_bitfield).unwrap();

        buffer
    }

    /// Unpacks a byte slice into a `SlowLinkAckPacket`.
    ///
    /// This method deserializes a byte slice into a `SlowLinkAckPacket` instance.
    /// It reads the packet_type, highest_packet_id, and packet_bitfield.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the serialized packet data
    ///
    /// # Returns
    ///
    /// * `Result<Self, &'static str>` - A Result containing either the deserialized packet or an error
    pub fn unpack(data: &[u8]) -> Result<Self, &'static str> {
        // Check if the data has at least the required size (1 + 8 + 8 bytes)
        const REQUIRED_SIZE: usize = 1 + 8 + 8;
        if data.len() < REQUIRED_SIZE {
            return Err("Data is too short to contain a valid acknowledge packet");
        }

        let mut cursor = Cursor::new(data);

        // Read packet_type
        let packet_type = match cursor.read_u8() {
            Ok(pt) => pt,
            Err(_) => return Err("Failed to read packet type"),
        };

        // Validate packet type
        match SlowLinkPacketType::try_from(packet_type) {
            Ok(SlowLinkPacketType::Acknowledge) => {}
            Ok(_) => return Err("Not an acknowledge packet"),
            Err(e) => return Err(e),
        }

        // Read highest_packet_id
        let highest_packet_id = match cursor.read_u64::<BigEndian>() {
            Ok(id) => id,
            Err(_) => return Err("Failed to read highest packet id"),
        };

        // Read packet_bitfield
        let packet_bitfield = match cursor.read_u64::<BigEndian>() {
            Ok(bitfield) => bitfield,
            Err(_) => return Err("Failed to read packet bitfield"),
        };

        Ok(Self {
            packet_type,
            highest_packet_id,
            packet_bitfield,
        })
    }
}

//=============================================================================
// SlowLinkPacket
//=============================================================================
/// An enum representing the different types of packets that can be sent through a SlowLink.
///
/// This enum provides a type-safe way to handle different packet variants.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SlowLinkPacket {
    Invalid,
    Acknowledge(SlowLinkAckPacket),
    Hello,
    Payload(SlowLinkPayloadPacket),
}

impl SlowLinkPacket {
    /// Unpacks a byte vector into a `SlowLinkPacket` of the correct type.
    ///
    /// This function examines the first byte of the data to determine the packet type,
    /// then calls the appropriate unpack method for that packet type.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte vector containing the serialized packet data
    ///
    /// # Returns
    ///
    /// * `SlowLinkPacket` - The unpacked packet as the appropriate variant
    pub fn unpack(data: Vec<u8>) -> SlowLinkPacket {
        if data.is_empty() {
            return SlowLinkPacket::Invalid;
        }

        // Check the first byte to determine packet type
        match SlowLinkPacketType::try_from(data[0]) {
            Ok(SlowLinkPacketType::Acknowledge) => match SlowLinkAckPacket::unpack(&data) {
                Ok(packet) => SlowLinkPacket::Acknowledge(packet),
                Err(_) => SlowLinkPacket::Invalid,
            },
            Ok(SlowLinkPacketType::Hello) => {
                // Hello packet is simple and has no additional data
                if data.len() == 1 && data[0] == SlowLinkPacketType::Hello as u8 {
                    SlowLinkPacket::Hello
                } else {
                    SlowLinkPacket::Invalid
                }
            }
            Ok(SlowLinkPacketType::Payload) => match SlowLinkPayloadPacket::unpack(&data) {
                Ok(packet) => SlowLinkPacket::Payload(packet),
                Err(_) => SlowLinkPacket::Invalid,
            },
            Err(_) => SlowLinkPacket::Invalid,
        }
    }
}
