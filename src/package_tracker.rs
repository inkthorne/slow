use crate::junction_id::JunctionId;
use crate::package::SlowPackage;
use crate::tracker::{PacketTracker, UpdateResult};
use std::collections::HashMap;

/// Tracks package information for Slow network communications.
///
/// This struct maintains a map of PacketTrackers for each unique junction ID,
/// enabling monitoring of received packages and detection of duplicates on a
/// per-sender basis.
pub struct SlowPackageTracker {
    /// Map of packet trackers for each sender junction
    packet_trackers: HashMap<JunctionId, PacketTracker>,
}

impl SlowPackageTracker {
    /// Creates a new `SlowPackageTracker` instance.
    ///
    /// # Returns
    ///
    /// A new `SlowPackageTracker` with an empty map of packet trackers.
    pub fn new() -> Self {
        SlowPackageTracker {
            packet_trackers: HashMap::new(),
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
        let tracker = self
            .packet_trackers
            .entry(package.sender_id().clone())
            .or_insert_with(PacketTracker::new);
        tracker.update(package.package_id() as u64)
    }
}
