use slow::junction::JunctionId;
use slow::link::{SlowLink, UnpackResult};
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
    let unpack_result = link.unpack(&packed_data);

    // Verify we got a Payload result with a valid index
    match unpack_result {
        UnpackResult::Payload(start_idx) => {
            // Extract and verify the package data
            let package_data = &packed_data[start_idx..];
            assert!(!package_data.is_empty());
            // let _package = SlowPackage::unpackage(package_data).unwrap();
            // let package_type = package.package_type().unwrap();
            // assert!(package_type == PackageType::Ping);
        }
        _ => panic!("Expected UnpackResult::Payload, got {:?}", unpack_result),
    }
}
