use crate::junction::JunctionId;
use bincode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ===========================================================================
// PackageType
// ===========================================================================

/// Represents the type of a package in the Slow network.
///
/// The `PackageType` enum defines the different types of packages that can be
/// transmitted in the Slow network. Each variant represents a specific type
/// of package.
#[derive(Serialize, Deserialize)]
pub enum PackageType {
    Ping,
    Pong,
    Json,
    Bin,
}

impl From<PackageType> for u8 {
    /// Converts a `PackageType` into a `u8`.
    ///
    /// This function maps each `PackageType` variant to a unique `u8` value.
    ///
    /// # Arguments
    ///
    /// * `package_type` - A `PackageType` instance.
    ///
    /// # Returns
    ///
    /// * `u8` - The corresponding `u8` value for the `PackageType`.
    fn from(package_type: PackageType) -> Self {
        match package_type {
            PackageType::Ping => 0,
            PackageType::Pong => 1,
            PackageType::Json => 2,
            PackageType::Bin => 3,
        }
    }
}

impl TryFrom<u8> for PackageType {
    type Error = ();

    /// Attempts to convert a `u8` into a `PackageType`.
    ///
    /// This function tries to map a `u8` value to a `PackageType` variant.
    /// If the value does not correspond to any variant, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `value` - A `u8` value.
    ///
    /// # Returns
    ///
    /// * `Result<PackageType, ()>` - A result containing the `PackageType`
    ///   if the conversion is successful, or an error if it fails.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PackageType::Ping),
            1 => Ok(PackageType::Pong),
            2 => Ok(PackageType::Json),
            3 => Ok(PackageType::Bin),
            _ => Err(()),
        }
    }
}

// ===========================================================================
// SlowPackageHeader
// ===========================================================================

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

    /// The type of data contained in the payload (see PayloadType).
    pub package_type: u8,

    /// An incrementing number that uniquely identifies a package from the specific sender.
    pub package_id: u32,

    /// The size of the payload in bytes.
    pub payload_size: u16,
}

// ===========================================================================
// SlowPackage
// ===========================================================================

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
    /// * `Self` - A `SlowPackage` instance.
    pub fn new_json_payload(recipient_id: JunctionId, sender_id: JunctionId, json: &Value) -> Self {
        let payload = serde_json::to_vec(json).unwrap();
        let header = SlowPackageHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            package_type: PackageType::Json.into(),
            package_id: 0,
            payload_size: payload.len() as u16,
        };

        SlowPackage { header, payload }
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
    /// * `Self` - A `SlowPackage` instance.
    pub fn new_bin_payload(recipient_id: JunctionId, sender_id: JunctionId, bin: &[u8]) -> Self {
        let payload = bin.to_vec();
        let header = SlowPackageHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            package_type: PackageType::Bin.into(),
            package_id: 0,
            payload_size: payload.len() as u16,
        };

        SlowPackage { header, payload }
    }

    /// Creates a new `SlowPackage` instance representing a Ping package.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A `JunctionId` representing the recipient.
    /// * `sender_id` - A `JunctionId` representing the sender.
    ///
    /// # Returns
    ///
    /// * `Self` - A `SlowPackage` instance.
    pub fn new_ping(recipient_id: JunctionId, sender_id: JunctionId) -> Self {
        let payload = Vec::new();
        let header = SlowPackageHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            package_type: PackageType::Ping.into(),
            package_id: 0,
            payload_size: payload.len() as u16,
        };

        SlowPackage { header, payload }
    }

    /// Creates a new `SlowPackage` instance representing a Pong package.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A `JunctionId` representing the recipient.
    /// * `sender_id` - A `JunctionId` representing the sender.
    ///
    /// # Returns
    ///
    /// * `Self` - A `SlowPackage` instance.
    pub fn new_pong(recipient_id: JunctionId, sender_id: JunctionId) -> Self {
        let payload = Vec::new();
        let header = SlowPackageHeader {
            recipient_id,
            sender_id,
            hop_count: 0,
            package_type: PackageType::Pong.into(),
            package_id: 0,
            payload_size: payload.len() as u16,
        };

        SlowPackage { header, payload }
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

    /// Returns the `package_type` from the header.
    ///
    /// # Returns
    ///
    /// * `PackageType` - The package type.
    pub fn get_package_type(&self) -> Result<PackageType, ()> {
        PackageType::try_from(self.header.package_type)
    }
}
