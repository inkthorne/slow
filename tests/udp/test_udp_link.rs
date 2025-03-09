use slow::junction::JunctionId;
use slow::link_packet::SlowLinkPacket;
use slow::package::SlowPackage;
use slow::udp::udp_link::SlowUdpLink;
use slow::udp::udp_socket::SlowUdpSocket;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_slow_udp_link() {
    // Create a UDP socket first
    let addr = SocketAddr::from_str("127.0.0.1:3000").unwrap();
    let socket = SlowUdpSocket::new(addr).await.unwrap();
    let socket = Arc::new(socket);

    // Create a new SlowLink instance with the socket
    let mut link = SlowUdpLink::new(addr, socket).unwrap();

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
    match packet {
        SlowLinkPacket::Payload(payload_packet) => {
            assert_eq!(payload_packet.packet_id, 1);
            assert_eq!(payload_packet.payload, package.pack(0));
        }
        _ => panic!("Unexpected packet type"),
    }
}
