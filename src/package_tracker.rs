use crate::package::SlowPackage;
use crate::tracker::{PacketTracker, UpdateResult};

/// Tracks package information for Slow network communications.
///
/// This struct wraps a PacketTracker to provide package-specific tracking
/// functionality, enabling monitoring of received packages and detection
/// of duplicates.
pub struct SlowPackageTracker {
    /// The underlying packet tracker used to track package IDs
    packet_tracker: PacketTracker,
}

impl SlowPackageTracker {
    /// Creates a new `SlowPackageTracker` instance.
    ///
    /// # Returns
    ///
    /// A new `SlowPackageTracker` with a fresh `PacketTracker`.
    pub fn new() -> Self {
        SlowPackageTracker {
            packet_tracker: PacketTracker::new(),
        }
    }

    /// Updates the package information with a new package.
    ///
    /// # Arguments
    ///
    /// * `package` - The SlowPackage to update.
    ///
    /// # Returns
    ///
    /// The result of the update operation. Returns `UpdateResult::Duplicate` if the package is a duplicate,
    /// `UpdateResult::Old` if the package is too old, or `UpdateResult::Success` if the package was
    /// successfully processed.
    pub fn update(&mut self, package: &SlowPackage) -> UpdateResult {
        self.packet_tracker.update(package.package_id() as u64)
    }

    /// Gets the highest package ID received.
    ///
    /// # Returns
    ///
    /// The highest package ID received.
    pub fn highest_package_id(&self) -> u64 {
        self.packet_tracker.highest_packet_id()
    }

    /// Gets the package bitfield used for tracking recent packages.
    ///
    /// # Returns
    ///
    /// The package bitfield.
    pub fn package_bitfield(&self) -> u64 {
        self.packet_tracker.packet_bitfield()
    }
}

impl Default for SlowPackageTracker {
    fn default() -> Self {
        Self::new()
    }
}
