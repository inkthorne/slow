use crate::link_packet::{SlowLinkAckPacket, SlowLinkPacket, SlowLinkPayloadPacket};
use crate::package::SlowPackage;
use crate::tracker::PacketTracker;
use crate::udp::udp_socket::SlowUdpSocket;
use std::net::SocketAddr;
use std::sync::Arc;

//=============================================================================
// UnpackResult
//=============================================================================
/// Represents the result of unpacking a packet.
///
/// This enum defines the possible outcomes of unpacking a packet.
#[derive(Debug, PartialEq)]
pub enum UnpackResult {
    /// Indicates that the packet was successfully unpacked and provides the starting index of payload data.
    Payload(usize),
    /// Indicates that the packet was a control packet.
    Control,
    /// Indicates that the packet was a duplicate packet and was discarded.
    Duplicate,
    /// Indicates that the packet was too old to be tracked and was discarded.
    Old,
    /// Indicates that the packet was invalid or malformed.
    Invalid,
}

//=============================================================================
// SlowUdpLink
//=============================================================================
/// A `SlowUdpLink` represents a direct connection between two junctions in the network.
///
/// This struct provides methods to create a new link and send packages through it.
/// It is designed to simplify direct communication between two specific junctions.
pub struct SlowUdpLink {
    /// The remote junction address.
    remote_address: SocketAddr,
    /// Counter for packets successfully sent through this link.
    packed_count: u64,
    /// Packet state tracking for received packets.
    unpacked_tracker: PacketTracker,
    /// The underlying UDP socket.
    socket: Arc<SlowUdpSocket>,
}

impl SlowUdpLink {
    /// Creates a new `SlowUdpLink` instance.
    ///
    /// # Arguments
    ///
    /// * `remote_address` - The address of the remote junction
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowUdpLink` or an error
    pub fn new(remote_address: SocketAddr, socket: Arc<SlowUdpSocket>) -> std::io::Result<Self> {
        Ok(Self {
            remote_address,
            packed_count: 0,
            unpacked_tracker: PacketTracker::new(),
            socket,
        })
    }

    /// Creates a packet buffer from a `SlowPackage`.
    ///
    /// This method puts the package into a buffer preceded by a SlowLinkPayloadPacket header.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` to pack
    ///
    /// # Returns
    ///
    /// * `Option<Vec<u8>>` - The packed buffer containing header and package data, or None if serialization fails
    pub fn pack(&mut self, package: &SlowPackage) -> Option<Vec<u8>> {
        self.packed_count += 1;

        let packet_id = self.packed_count;
        let data = package.pack(package.package_id());
        let payload_packet = SlowLinkPayloadPacket::new(packet_id, data);
        let packed = payload_packet.pack();

        Some(packed)
    }

    /// Unpacks a received packet and determine its type.
    ///
    /// This method analyzes the provided byte slice to determine whether it contains
    /// a payload packet or an acknowledgment packet, then processes it accordingly.
    ///
    /// # Arguments
    ///
    /// * `data` - The received packet as a byte slice
    ///
    /// # Returns
    ///
    /// * `SlowLinkPacket` - The unpacked packet
    pub fn unpack(&mut self, data: &[u8]) -> SlowLinkPacket {
        if data.is_empty() {
            return SlowLinkPacket::Invalid;
        }

        let packet = SlowLinkPacket::unpack(data.to_vec());

        match packet {
            SlowLinkPacket::Payload(ref payload_packet) => self.process_payload(&payload_packet),
            SlowLinkPacket::Acknowledge(ref ack_packet) => self.process_ack(&ack_packet),
            _ => {}
        };

        packet
    }

    /// Process a payload packet by updating the packet tracker.
    ///
    /// # Arguments
    ///
    /// * `payload_packet` - The received payload packet
    fn process_payload(&mut self, payload_packet: &SlowLinkPayloadPacket) {
        // Update the packet tracker with this new packet ID
        self.unpacked_tracker.update(payload_packet.packet_id);
    }

    /// Process an acknowledgment packet.
    ///
    /// # Arguments
    ///
    /// * `_ack_packet` - The received acknowledgment packet
    fn process_ack(&self, _ack_packet: &SlowLinkAckPacket) {}

    /// Sends data over the UDP link.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send
    ///
    /// # Returns
    /// The number of bytes sent
    ///
    /// # Errors
    /// Returns an error if the transmission fails
    pub async fn send(&self, data: &[u8]) -> std::io::Result<usize> {
        self.socket.send(data, &self.remote_address).await
    }

    /// Receives data from the UDP link.
    ///
    /// # Arguments
    /// * `buffer` - Buffer to store the received data
    ///
    /// # Returns
    /// The number of bytes read into the buffer
    ///
    /// # Errors
    /// Returns an error if reading fails
    pub async fn receive(&self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        // Extract just the size from the tuple returned by socket.receive
        // let (size, _) = self.socket.receive(buffer).await?;
        // Ok(size)
        Ok(0)
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
    pub fn packed_count(&self) -> u64 {
        self.packed_count
    }
}
