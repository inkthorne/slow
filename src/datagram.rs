use bincode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct SlowDatagramHeader {
    pub to: u16,
    pub hops_remaining: u16,
    pub size: u16,
}

pub struct SlowDatagram {
    pub header: SlowDatagramHeader,
    pub data: Vec<u8>,
}

impl SlowDatagram {
    /// Creates a new `SlowDatagram` instance.
    ///
    /// # Arguments
    ///
    /// * `to` - A `u16` representing the recipient.
    /// * `json` - A reference to a `Value` representing the JSON data.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - An optional `SlowDatagram` instance.
    pub fn new(to: u16, json: &Value) -> Option<Self> {
        let data = serde_json::to_vec(json).ok()?;
        let header = SlowDatagramHeader {
            to,
            hops_remaining: 4,
            size: data.len() as u16,
        };
        Some(SlowDatagram { header, data })
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
        let header_data = &data[..std::mem::size_of::<SlowDatagramHeader>()];
        let header: SlowDatagramHeader = bincode::deserialize(header_data).ok()?;
        let json_data = &data[std::mem::size_of::<SlowDatagramHeader>()..];
        if header.size as usize == json_data.len() {
            Some(SlowDatagram {
                header,
                data: json_data.to_vec(),
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
        package.extend_from_slice(&self.data);
        package
    }

    /// Returns the `data` as a JSON value.
    ///
    /// # Returns
    ///
    /// * `Option<Value>` - An optional `Value` representing the JSON data.
    pub fn get_json(&self) -> Option<Value> {
        serde_json::from_slice(&self.data).ok()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new() {
        let json_data = json!({"key": "value"});
        let datagram = SlowDatagram::new(1, &json_data).unwrap();
        assert_eq!(datagram.header.to, 1);
        assert_eq!(
            datagram.header.size,
            serde_json::to_vec(&json_data).unwrap().len() as u16
        );
        assert_eq!(datagram.get_json().unwrap(), json_data);
    }

    #[test]
    fn test_peek_header() {
        let json_data = json!({"key": "value"});
        let datagram = SlowDatagram::new(1, &json_data).unwrap();
        let packaged_data = datagram.package();
        let header = SlowDatagram::peek_header(&packaged_data).unwrap();
        assert_eq!(header.to, 1);
        assert_eq!(
            header.size,
            serde_json::to_vec(&json_data).unwrap().len() as u16
        );
    }

    #[test]
    fn test_unpackage() {
        let json_data = json!({"key": "value"});
        let datagram = SlowDatagram::new(1, &json_data).unwrap();
        let packaged_data = datagram.package();
        let unpackaged_datagram = SlowDatagram::unpackage(&packaged_data).unwrap();
        assert_eq!(unpackaged_datagram.header.to, 1);
        assert_eq!(
            unpackaged_datagram.header.size,
            serde_json::to_vec(&json_data).unwrap().len() as u16
        );
        assert_eq!(unpackaged_datagram.get_json().unwrap(), json_data);
    }

    #[test]
    fn test_package() {
        let json_data = json!({"key": "value"});
        let datagram = SlowDatagram::new(1, &json_data).unwrap();
        let packaged_data = datagram.package();
        let header_size = std::mem::size_of::<SlowDatagramHeader>();
        assert_eq!(
            &packaged_data[..header_size],
            &bincode::serialize(&datagram.header).unwrap()[..]
        );
        assert_eq!(&packaged_data[header_size..], &datagram.data[..]);
    }

    #[test]
    fn test_get_json() {
        let json_data = json!({"key": "value"});
        let datagram = SlowDatagram::new(1, &json_data).unwrap();
        assert_eq!(datagram.get_json().unwrap(), json_data);
    }

    #[test]
    fn test_decrement_hops() {
        let json_data = json!({"key": "value"});
        let mut datagram = SlowDatagram::new(1, &json_data).unwrap();
        assert_eq!(datagram.header.hops_remaining, 4);
        assert!(datagram.decrement_hops());
        assert_eq!(datagram.header.hops_remaining, 3);
        assert!(datagram.decrement_hops());
        assert_eq!(datagram.header.hops_remaining, 2);
        assert!(datagram.decrement_hops());
        assert_eq!(datagram.header.hops_remaining, 1);
        assert!(!datagram.decrement_hops());
        assert_eq!(datagram.header.hops_remaining, 0);
        assert!(!datagram.decrement_hops());
    }
}
