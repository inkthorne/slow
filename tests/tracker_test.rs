use slow::tracker::PacketTracker;

#[test]
fn test_highest_packet_id_update() {
    let mut tracker = PacketTracker::new();

    // Initial state should be 0
    assert_eq!(tracker.highest_packet_id(), 0);

    // Update with a higher packet ID
    assert!(tracker.update(5));
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with a lower packet ID should not change highest_packet_id
    assert!(tracker.update(3)); // Returns true because it's tracked in bitfield
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with a duplicate lower packet ID should not change highest_packet_id
    assert!(!tracker.update(3)); // Returns false because it's tracked in bitfield
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with same packet ID should return false
    assert!(!tracker.update(5));
    assert_eq!(tracker.highest_packet_id(), 5);

    // Update with same packet ID should return false
    assert!(tracker.update(64)); // Returns true because it's a new packet ID
    assert_eq!(tracker.highest_packet_id(), 64);

    // Update with same packet ID should return false
    assert!(!tracker.update(5)); // Returns false because packet exists in the bitfield
    assert_eq!(tracker.highest_packet_id(), 64);

    // Update with much higher packet ID
    assert!(tracker.update(100));
    assert_eq!(tracker.highest_packet_id(), 100);

    // Update with much lower packet ID
    assert!(!tracker.update(1)); // Returns false because it's too old to be tracked
    assert_eq!(tracker.highest_packet_id(), 100);
}
