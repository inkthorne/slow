use crate::junction::JunctionId;
use bincode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct SlowDatagramHeader {
    pub recipient_id: JunctionId, // changed from String to JunctionId
    pub hops_remaining: u16,
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
    /// * `recipient_id` - A `String` representing the recipient.
    /// * `json` - A reference to a `Value` representing the JSON data.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowDatagram` instance.
    pub fn new(recipient_id: String, json: &Value) -> Option<Self> {
        let payload = serde_json::to_vec(json).ok()?;
        let header = SlowDatagramHeader {
            recipient_id: JunctionId::new(&recipient_id), // updated construction
            hops_remaining: 4,
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
        let header_data = bincode::serialize(&self.header).unwrap();
        let mut package = Vec::new();
        package.extend_from_slice(&header_data);
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

    /// Decrements the `hops_remaining` field by 1.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if there are hops remaining, `false` otherwise.
    pub fn decrement_hops(&mut self) -> bool {
        if self.header.hops_remaining > 0 {
            self.header.hops_remaining -= 1;
        }
        self.header.hops_remaining > 0
    }

    /// Returns the `recipient_id` from the header.
    ///
    /// # Returns
    ///
    /// * `&JunctionId` - The recipient ID.
    pub fn get_recipient_id(&self) -> &JunctionId {
        &self.header.recipient_id
    }
}
