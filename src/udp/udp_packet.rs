use std::convert::TryFrom;

//=============================================================================
// SlowUdpPacketType
//=============================================================================
/// Represents the type of UDP packet in the Slow protocol
///
/// - `Data`: Packet containing actual data payload
/// - `Ack`: Acknowledgment packet confirming receipt of Data packets
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SlowUdpPacketType {
    Data = 0,
    Ack = 1,
}

impl From<SlowUdpPacketType> for u8 {
    fn from(packet_type: SlowUdpPacketType) -> Self {
        packet_type as u8
    }
}

impl TryFrom<u8> for SlowUdpPacketType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SlowUdpPacketType::Data),
            1 => Ok(SlowUdpPacketType::Ack),
            _ => Err(()),
        }
    }
}

//=============================================================================
// SlowUdpPacket
//=============================================================================
/// Represents a UDP packet in the Slow protocol
///
/// Can be either a Data packet or an Ack packet
#[derive(Debug)]
pub enum SlowUdpPacket {
    Data(SlowUdpDataPacket),
    Ack(SlowUdpAckPacket),
}

impl SlowUdpPacket {
    /// Deserializes a byte buffer into the appropriate packet type
    ///
    /// # Arguments
    /// * `buffer` - The byte slice to deserialize from
    ///
    /// # Returns
    /// * `Ok(SlowUdpPacket)` - Successfully deserialized packet (either Data or Ack)
    /// * `Err(())` - Failed to deserialize (invalid format)
    pub fn unpack(buffer: &[u8]) -> Result<Self, ()> {
        if buffer.len() < 1 {
            return Err(());
        }

        match SlowUdpPacketType::try_from(buffer[0])? {
            SlowUdpPacketType::Data => Ok(SlowUdpPacket::Data(SlowUdpDataPacket::unpack(buffer)?)),
            SlowUdpPacketType::Ack => Ok(SlowUdpPacket::Ack(SlowUdpAckPacket::unpack(buffer)?)),
        }
    }
}

//=============================================================================
// SlowUdpAckPacket
//=============================================================================
/// Represents a UDP acknowledgment packet in the Slow protocol
///
/// Contains packet ID and a bitfield for tracking which packets have been received
#[derive(Debug)]
pub struct SlowUdpAckPacket {
    pub packet_type: SlowUdpPacketType,
    pub packet_id: u32,
    pub packet_bitfield: u64,
}

impl SlowUdpAckPacket {
    /// Serializes the acknowledgment packet into a byte vector
    ///
    /// The byte structure is:
    /// - 1 byte: packet type
    /// - 4 bytes: packet ID (big endian)
    /// - 8 bytes: packet bitfield (big endian)
    ///
    /// # Returns
    /// A vector of bytes representing the serialized packet
    pub fn pack(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(13); // 1 + 4 + 8
        buffer.push(self.packet_type.into());
        buffer.extend_from_slice(&self.packet_id.to_be_bytes());
        buffer.extend_from_slice(&self.packet_bitfield.to_be_bytes());
        buffer
    }

    /// Deserializes a byte buffer into an acknowledgment packet
    ///
    /// # Arguments
    /// * `buffer` - The byte slice to deserialize from
    ///
    /// # Returns
    /// * `Ok(SlowUdpAckPacket)` - Successfully deserialized packet
    /// * `Err(())` - Failed to deserialize (invalid format or incorrect packet type)
    pub fn unpack(buffer: &[u8]) -> Result<Self, ()> {
        if buffer.len() < 13 {
            return Err(());
        }

        let packet_type = SlowUdpPacketType::try_from(buffer[0])?;
        if packet_type != SlowUdpPacketType::Ack {
            return Err(());
        }

        let packet_id = u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        let packet_bitfield = u64::from_be_bytes([
            buffer[5], buffer[6], buffer[7], buffer[8], buffer[9], buffer[10], buffer[11],
            buffer[12],
        ]);

        Ok(Self {
            packet_type,
            packet_id,
            packet_bitfield,
        })
    }
}

//=============================================================================
// SlowUdpDataPacket
//=============================================================================
/// Represents a UDP data packet in the Slow protocol
///
/// Contains packet metadata and payload data for transmission over UDP
#[derive(Debug)]
pub struct SlowUdpDataPacket {
    pub packet_type: SlowUdpPacketType,
    pub packet_id: u32,
    pub packet_index: u16,
    pub packet_count: u16,
    pub packet_data: Vec<u8>,
}

impl SlowUdpDataPacket {
    /// Serializes the data packet into a byte vector
    ///
    /// The byte structure is:
    /// - 1 byte: packet type
    /// - 4 bytes: packet ID (big endian)
    /// - 2 bytes: packet index (big endian)
    /// - 2 bytes: packet count (big endian)
    /// - remainder: packet data
    ///
    /// # Returns
    /// A vector of bytes representing the serialized packet
    pub fn pack(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(9 + self.packet_data.len()); // 1 + 4 + 2 + 2 + data
        buffer.push(self.packet_type.into());
        buffer.extend_from_slice(&self.packet_id.to_be_bytes());
        buffer.extend_from_slice(&self.packet_index.to_be_bytes());
        buffer.extend_from_slice(&self.packet_count.to_be_bytes());
        buffer.extend_from_slice(&self.packet_data);
        buffer
    }

    /// Deserializes a byte buffer into a data packet
    ///
    /// # Arguments
    /// * `buffer` - The byte slice to deserialize from
    ///
    /// # Returns
    /// * `Ok(SlowUdpDataPacket)` - Successfully deserialized packet
    /// * `Err(())` - Failed to deserialize (invalid format or incorrect packet type)
    pub fn unpack(buffer: &[u8]) -> Result<Self, ()> {
        if buffer.len() < 9 {
            // Minimum size is 9 bytes (header without data)
            return Err(());
        }

        let packet_type = SlowUdpPacketType::try_from(buffer[0])?;
        if packet_type != SlowUdpPacketType::Data {
            return Err(());
        }

        let packet_id = u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);
        let packet_index = u16::from_be_bytes([buffer[5], buffer[6]]);
        let packet_count = u16::from_be_bytes([buffer[7], buffer[8]]);
        let packet_data = buffer[9..].to_vec();

        Ok(Self {
            packet_type,
            packet_id,
            packet_index,
            packet_count,
            packet_data,
        })
    }
}
