use slow::tracker::{PacketTracker, UpdateResult};

#[test]
fn test_highest_packet_id_update() {
    let mut tracker = PacketTracker::new();

    // Initial state should be 0
    assert_eq!(tracker.highest_packet_id(), 0);

    // Update with a higher packet ID
    assert_eq!(tracker.update(5), UpdateResult::Success);
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with a lower packet ID should not change highest_packet_id
    assert_eq!(tracker.update(3), UpdateResult::Success); // Tracked in bitfield
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with a duplicate lower packet ID should not change highest_packet_id
    assert_eq!(tracker.update(3), UpdateResult::Duplicate); // Already tracked in bitfield
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with same packet ID should return Duplicate
    assert_eq!(tracker.update(5), UpdateResult::Duplicate);
    assert_eq!(tracker.highest_packet_id(), 5);

    assert_eq!(tracker.update(64), UpdateResult::Success); // New packet ID
    assert_eq!(tracker.highest_packet_id(), 64);

    // Update with same packet ID should return Duplicate
    assert_eq!(tracker.update(5), UpdateResult::Duplicate);
    assert_eq!(tracker.highest_packet_id(), 64);

    // Update with much higher packet ID
    assert_eq!(tracker.update(100), UpdateResult::Success);
    assert_eq!(tracker.highest_packet_id(), 100);

    // Update with much lower packet ID
    assert_eq!(tracker.update(1), UpdateResult::Old); // Too old to be in bitfield
    assert_eq!(tracker.highest_packet_id(), 100);
}
