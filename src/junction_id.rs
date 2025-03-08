/// A `JunctionId` represents the unique identifier for a network junction.
///
/// This struct provides methods to create a new junction ID and format it for display.
#[derive(Clone, Hash, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JunctionId {
    /// The unique identifier for the junction.
    id: String,
}

impl JunctionId {
    /// Creates a new `JunctionId` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice that holds the ID.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of `JunctionId`.
    pub fn new(id: &str) -> Self {
        JunctionId { id: id.to_string() }
    }

    /// Manually serializes the `JunctionId` into a byte vector.
    ///
    /// The format is:
    /// - 2 bytes: Length of the ID string as u16 in little-endian
    /// - N bytes: The ID string as UTF-8 bytes
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The serialized `JunctionId` as a byte vector.
    pub fn pack(&self) -> Vec<u8> {
        let id_bytes = self.id.as_bytes();
        let id_len = id_bytes.len() as u16;

        let mut buffer = Vec::with_capacity(2 + id_bytes.len());

        // Write the length of the ID string as u16 (little-endian)
        buffer.extend_from_slice(&id_len.to_le_bytes());

        // Write the ID string bytes
        buffer.extend_from_slice(id_bytes);

        buffer
    }

    /// Manually deserializes a byte slice into a `JunctionId` instance.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice containing the serialized `JunctionId` data.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - A new `JunctionId` instance if deserialization is successful, None otherwise.
    pub fn unpack(data: &[u8]) -> Option<Self> {
        // Check if we have at least 2 bytes for the length
        if data.len() < 2 {
            return None;
        }

        // Read string length (u16)
        let id_len = u16::from_le_bytes([data[0], data[1]]) as usize;

        // Check if we have enough bytes for the ID string
        if data.len() < 2 + id_len {
            return None;
        }

        // Read and convert ID string bytes to a UTF-8 string
        match std::str::from_utf8(&data[2..2 + id_len]) {
            Ok(id_str) => Some(JunctionId::new(id_str)),
            Err(_) => None,
        }
    }
}

impl std::fmt::Display for JunctionId {
    /// Formats the `JunctionId` for display.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a `std::fmt::Formatter`.
    ///
    /// # Returns
    ///
    /// * `std::fmt::Result` - The result of the formatting operation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
