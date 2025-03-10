use crate::junction::JunctionId;
use crate::package::SlowPackage;
use crate::tcp::tcp_link::SlowLinkId;
use crate::tracker::PacketTracker;
use crate::tracker::UpdateResult;
use std::collections::HashMap;

//=============================================================================
// SlowTcpLinkStats
//=============================================================================

struct SlowTcpLinkStats {
    /// Packets successfully received or forwarded
    valid_packet_count: u64,

    /// Old or duplicate packets that were dropped
    invalid_packet_count: u64,
}

impl SlowTcpLinkStats {
    /// Creates a new `SlowTcpLinkStats` with zero counters.
    pub fn new() -> Self {
        SlowTcpLinkStats {
            valid_packet_count: 0,
            invalid_packet_count: 0,
        }
    }
}

//=============================================================================
// SlowTcpRouteStats
//=============================================================================

struct SlowTcpRouteStats {
    /// The link statistics for packets received from this junction
    /// The key is the link ID
    link_stats: HashMap<u32, SlowTcpLinkStats>,

    /// Tracks packet receipt information from this junction
    packet_tracker: PacketTracker,
}

impl SlowTcpRouteStats {
    /// Creates a new `SlowTcpRouter` instance.
    fn new() -> Self {
        SlowTcpRouteStats {
            link_stats: HashMap::new(),
            packet_tracker: PacketTracker::new(),
        }
    }

    /// Returns the link ID with the highest valid_packet_count
    ///
    /// # Returns
    ///
    /// * `Option<u32>` - The link ID with the highest valid_packet_count, or None if there are no links
    fn get_best_link(&self) -> Option<u32> {
        self.link_stats
            .iter()
            .max_by_key(|&(_, stats)| stats.valid_packet_count)
            .map(|(link_id, _)| *link_id)
    }

    /// Updates the packet tracker with the package ID from the provided SlowPackage
    ///
    /// # Arguments
    ///
    /// * `package` - A reference to a SlowPackage
    ///
    /// # Returns
    ///
    /// The result of the update operation
    fn update(&mut self, package: &SlowPackage, link_id: SlowLinkId) -> UpdateResult {
        // Convert u32 package_id to u64 for the packet tracker
        let packet_id = package.package_id() as u64;
        let result = self.packet_tracker.update(packet_id);

        // Get or create link stats for this link_id
        let stats = self
            .link_stats
            .entry(link_id)
            .or_insert_with(SlowTcpLinkStats::new);

        // Update link statistics based on the result
        match result {
            UpdateResult::Success => {
                stats.valid_packet_count += 1;
            }
            UpdateResult::Duplicate | UpdateResult::Old => {
                stats.invalid_packet_count += 1;
            }
        }

        result
    }
}

//=============================================================================
// SlowTcpRouter
//=============================================================================

pub struct SlowTcpRouter {
    /// The route statistics for each junction
    /// The key is the JunctionId of the sender
    route_stats: HashMap<JunctionId, SlowTcpRouteStats>,
}

impl SlowTcpRouter {
    /// Creates a new `SlowTcpRouter` instance.
    pub fn new() -> Self {
        SlowTcpRouter {
            route_stats: HashMap::new(),
        }
    }

    /// Updates the route statistics for a package and link.
    ///
    /// # Arguments
    ///
    /// * `package` - A reference to a SlowPackage
    /// * `link_id` - The ID of the link that received the package
    ///
    /// # Returns
    ///
    /// The result of the update operation
    pub fn update(&mut self, package: &SlowPackage, link_id: SlowLinkId) -> UpdateResult {
        let sender_id = package.sender_id().clone();

        // Get or create route stats for this sender
        let stats = self
            .route_stats
            .entry(sender_id)
            .or_insert_with(SlowTcpRouteStats::new);

        // Update the route stats with the package and link_id
        stats.update(package, link_id)
    }

    /// Returns the best link for a junction.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The JunctionId to get the best link for
    ///
    /// # Returns
    ///
    /// * `Option<u32>` - The ID of the best link, or None if no link is found
    pub fn get_best_link(&self, junction_id: &JunctionId) -> Option<u32> {
        self.route_stats
            .get(junction_id)
            .and_then(|stats| stats.get_best_link())
    }
}
