use slow::link_packet::SlowLinkPayloadPacket;

#[test]
fn test_linke_payload_packet_pack_unpack() {
    // Create a test payload packet
    let packet_id = 42;
    let test_payload = b"Hello, SlowLink!".to_vec();
    let packet = SlowLinkPayloadPacket::new(packet_id, test_payload.clone());

    // Pack the packet
    let packed_data = packet.pack();

    // Verify the packed data is not empty
    assert!(!packed_data.is_empty());

    // Verify the packed data size is correct (1 byte type + 8 bytes id + 2 bytes size + payload)
    assert_eq!(packed_data.len(), 1 + 8 + 2 + test_payload.len());

    // Unpack the data
    let unpacked_packet =
        SlowLinkPayloadPacket::unpack(&packed_data).expect("Failed to unpack data");

    // Verify the unpacked packet matches the original
    assert_eq!(unpacked_packet, packet);
}

#[test]
fn test_payload_packet_invalid_data() {
    // Test with empty data
    let result = SlowLinkPayloadPacket::unpack(&[]);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Data is too short to contain a valid packet header"
    );

    // Test with incomplete header
    let result = SlowLinkPayloadPacket::unpack(&[2, 0, 1]); // Just 3 bytes
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Data is too short to contain a valid packet header"
    );

    // Test with invalid packet type
    let mut invalid_data = vec![255, 0, 0, 0, 0, 0, 0, 0, 42, 0, 5];
    invalid_data.extend_from_slice(&[1, 2, 3, 4, 5]);
    let result = SlowLinkPayloadPacket::unpack(&invalid_data);
    assert!(result.is_err());

    // Test with insufficient payload data
    let mut insufficient_data = vec![2, 0, 0, 0, 0, 0, 0, 0, 42, 0, 10]; // Payload size 10
    insufficient_data.extend_from_slice(&[1, 2, 3]); // Only 3 bytes
    let result = SlowLinkPayloadPacket::unpack(&insufficient_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Data is too short to contain the specified payload"
    );
}
