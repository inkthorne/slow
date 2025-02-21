use crate::junction::JunctionId;
use bincode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the header of a SlowPackage.
///
/// The header contains metadata about the package, such as the recipient ID,
/// sender ID, hop count, and payload size.
#[derive(Serialize, Deserialize)]
pub struct SlowPackageHeader {
    /// The ID of the recipient junction.
    pub recipient_id: JunctionId,

    /// The ID of the sender junction.
    pub sender_id: JunctionId,

    /// The number of hops the package has taken.
    pub hop_count: u8,

    /// The type of data contained in the payload: binary = 0, JSON = 1.
    pub payload_type: u8,

    /// An incrementing number that uniquely identifies a package from the specific sender.
    pub package_id: u32,

    /// The size of the payload in bytes.
    pub payload_size: u16,
}

/// Represents a package in the Slow network.
///
/// A SlowPackage consists of a header and a payload. The header contains metadata
/// about the package, while the payload contains the actual data being transmitted.
pub struct SlowPackage {
    /// The header of the package containing metadata.
    pub header: SlowPackageHeader,

    /// The payload of the package containing the actual data.
    pub payload: Vec<u8>,
}

impl SlowPackage {
    /// Creates a new `SlowPackage` instance.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A `JunctionId` representing the recipient.
    /// * `sender_id` - A `JunctionId` representing the sender.
    /// * `json` - A reference to a `Value` representing the JSON payload.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowPackage` instance.
    pub fn new_json_payload(
        recipient_id: JunctionId,
        sender_id: JunctionId,
        json: &Value,
    ) -> Option<Self> {
        let payload = serde_json::to_vec(json).ok()?;
        let header = SlowPackageHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            payload_type: 1,
            package_id: 0,
            payload_size: payload.len() as u16,
        };

        Some(SlowPackage { header, payload })
    }

    /// Creates a new `SlowPackage` instance.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A `JunctionId` representing the recipient.
    /// * `sender_id` - A `JunctionId` representing the sender.
    /// * `bin` - A reference to a slice representing the binary payload.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowPackage` instance.
    pub fn new_bin_payload(
        recipient_id: JunctionId,
        sender_id: JunctionId,
        bin: &[u8],
    ) -> Option<Self> {
        let payload = bin.to_vec();
        let header = SlowPackageHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            payload_type: 0,
            package_id: 0,
            payload_size: payload.len() as u16,
        };

        Some(SlowPackage { header, payload })
    }

    /// Unpackages a byte slice into a `SlowPackage`.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice containing the package data.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowPackage` instance.
    ///
    /// This function deserializes the header using `bincode` and checks the payload size.
    pub fn unpackage(data: &[u8]) -> Option<Self> {
        let mut cursor = std::io::Cursor::new(data);
        let header: SlowPackageHeader = bincode::deserialize_from(&mut cursor).ok()?;
        let header_size = cursor.position() as usize;
        let payload = &data[header_size..];

        if header.payload_size as usize == payload.len() {
            let package = SlowPackage {
                header,
                payload: payload.to_vec(),
            };

            return Some(package);
        }

        None
    }

    /// Packages the `SlowPackage` into a byte vector.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - A byte vector containing the packaged package.
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
    pub fn get_json_payload(&self) -> Option<Value> {
        serde_json::from_slice(&self.payload).ok()
    }

    /// Increments the `hop_count` field by 1.
    ///
    /// # Returns
    ///
    /// * `u16` - The new value of `hop_count`.
    pub fn increment_hops(&mut self) -> u8 {
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
    pub fn get_hop_count(&self) -> u8 {
        self.header.hop_count
    }

    /// Sets the `package_id` field.
    ///
    /// # Arguments
    ///
    /// * `package_id` - A `u32` representing the new package ID.
    pub fn set_package_id(&mut self, package_id: u32) {
        self.header.package_id = package_id;
    }

    /// Returns the `package_id` from the header.
    ///
    /// # Returns
    ///
    /// * `u32` - The package ID.
    pub fn get_package_id(&self) -> u32 {
        self.header.package_id
    }
}
