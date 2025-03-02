use slow::junction::JunctionId;
use slow::link::SlowLink;
use slow::link_packet::SlowLinkPacket;
use slow::package::SlowPackage;
use std::net::SocketAddr;
use std::str::FromStr;

#[test]
fn test_slow_link_basic() {
    // Create a test socket address
    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();

    // Create a new SlowLink instance
    let mut link = SlowLink::new(addr).unwrap();

    // Verify the initial state
    assert_eq!(link.remote_address(), addr);
    assert_eq!(link.packed_count(), 0);

    // Create and pack a test package
    let sender_id = JunctionId::new("sender");
    let recipient_id = JunctionId::new("recipient");
    let package = SlowPackage::new_ping(recipient_id, sender_id);

    // Pack the package and verify success
    let packed_data = link.pack(&package);
    assert!(packed_data.is_some());
    assert_eq!(link.packed_count(), 1);

    // Unpack the data and verify successful unpacking
    let packed_data = packed_data.unwrap();
    let packet = link.unpack(&packed_data);
    assert!(packet != SlowLinkPacket::Invalid);
}
