use crate::junction::JunctionId;
use bincode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct SlowDatagramHeader {
    pub recipient_id: JunctionId,
    pub sender_id: JunctionId,
    pub hop_count: u16,
    pub payload_size: u16,
}

pub struct SlowDatagram {
    pub header: SlowDatagramHeader,
    pub payload: Vec<u8>,
}

impl SlowDatagram {
    /// Creates a new `SlowDatagram` instance.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A `JunctionId` representing the recipient.
    /// * `sender_id` - A `JunctionId` representing the sender.
    /// * `json` - A reference to a `Value` representing the JSON data.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowDatagram` instance.
    pub fn new(recipient_id: JunctionId, sender_id: JunctionId, json: &Value) -> Option<Self> {
        let payload = serde_json::to_vec(json).ok()?;
        let header = SlowDatagramHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            payload_size: payload.len() as u16,
        };
        Some(SlowDatagram { header, payload })
    }

    /// Peeks at the header of a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice containing the datagram data.
    ///
    /// # Returns
    ///
    /// * `Option<SlowDatagramHeader>` - An optional `SlowDatagramHeader`.
    pub fn peek_header(data: &[u8]) -> Option<SlowDatagramHeader> {
        let header_data = &data[..std::mem::size_of::<SlowDatagramHeader>()];
        bincode::deserialize(header_data).ok()
    }

    /// Unpackages a byte slice into a `SlowDatagram`.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice containing the datagram data.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowDatagram` instance.
    pub fn unpackage(data: &[u8]) -> Option<Self> {
        let mut cursor = std::io::Cursor::new(data);
        let header: SlowDatagramHeader = bincode::deserialize_from(&mut cursor).ok()?;
        let header_size = cursor.position() as usize;
        let json_data = &data[header_size..];
        if header.payload_size as usize == json_data.len() {
            Some(SlowDatagram {
                header,
                payload: json_data.to_vec(),
            })
        } else {
            None
        }
    }

    /// Packages the `SlowDatagram` into a byte vector.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - A byte vector containing the packaged datagram.
    pub fn package(&self) -> Vec<u8> {
        let mut package = Vec::new();
        bincode::serialize_into(&mut package, &self.header).unwrap();
        package.extend_from_slice(&self.payload);
        package
    }

    /// Returns the `payload` as a JSON value.
    ///
    /// # Returns
    ///
    /// * `Option<Value>` - An optional `Value` representing the JSON data.
    pub fn get_json(&self) -> Option<Value> {
        serde_json::from_slice(&self.payload).ok()
    }

    /// Increments the `hop_count` field by 1.
    ///
    /// # Returns
    ///
    /// * `u16` - The new value of `hop_count`.
    pub fn increment_hops(&mut self) -> u16 {
        self.header.hop_count += 1;
        self.header.hop_count
    }

    /// Returns the `recipient_id` from the header.
    ///
    /// # Returns
    ///
    /// * `&JunctionId` - The recipient ID.
    pub fn get_recipient_id(&self) -> &JunctionId {
        &self.header.recipient_id
    }

    /// Returns the `sender_id` from the header.
    ///
    /// # Returns
    ///
    /// * `&JunctionId` - The sender ID.
    pub fn get_sender_id(&self) -> &JunctionId {
        &self.header.sender_id
    }

    /// Returns the `hop_count` from the header.
    ///
    /// # Returns
    ///
    /// * `u16` - The hop count.
    pub fn get_hop_count(&self) -> u16 {
        self.header.hop_count
    }
}
