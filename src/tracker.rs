/// Tracks packet receipt information in Link connections.
///
/// This struct monitors the highest packet ID received and maintains a bitfield
/// to efficiently track recently received packets, enabling duplicate detection
/// and reliable packet delivery confirmation.
pub struct PacketTracker {
    /// The ID of the highest packet received.
    highest_packet_id: u64,

    /// A bitfield used to track recently received packets relative to highest_packet_id.
    packet_bitfield: u64,
}

impl PacketTracker {
    /// Creates a new `PacketTracker`.
    ///
    /// # Returns
    ///
    /// A new instance of `PacketTracker` with default values.
    pub fn new() -> Self {
        PacketTracker {
            highest_packet_id: 0,
            packet_bitfield: 0,
        }
    }

    /// Updates the packet information with a new packet ID.
    ///
    /// # Arguments
    ///
    /// * `packet_id` - The ID of the packet to update.
    ///
    /// # Returns
    ///
    /// `true` if the packet information was updated successfully, `false` otherwise.
    pub fn update(&mut self, packet_id: u64) -> bool {
        let shift = packet_id as i64 - self.highest_packet_id as i64;

        // If the packet is the same ID we already received or it's too old, ignore it
        if shift == 0 || shift < -63 {
            return false;
        }

        // If the packet is older than the highest packet ID but within the bitfield range
        if shift < 0 {
            let mask = 1 << -shift;

            // Check if we've already received this packet
            if self.packet_bitfield & mask != 0 {
                return false;
            }

            // Mark this packet as received
            self.packet_bitfield |= mask;
            return true;
        }

        // If the packet is newer than the highest packet ID
        if shift > 63 {
            // Reset the bitfield and set the lowest bit
            self.packet_bitfield = 0;
            self.packet_bitfield |= 1;
        } else {
            // Shift the bitfield and set the lowest bit
            self.packet_bitfield <<= shift;
            self.packet_bitfield |= 1;
        }

        // Update the highest packet ID
        self.highest_packet_id = packet_id;

        true
    }

    /// Gets the highest packet ID received.
    ///
    /// # Returns
    ///
    /// The highest packet ID received.
    pub fn highest_packet_id(&self) -> u64 {
        self.highest_packet_id
    }

    /// Gets the packet bitfield.
    ///
    /// # Returns
    ///
    /// The packet bitfield.
    pub fn packet_bitfield(&self) -> u64 {
        self.packet_bitfield
    }
}
