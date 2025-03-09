use serde_json::json;
use slow::junction::JunctionId;
use slow::package::{PackageType, SlowPackage};

#[test]
fn test_package_new_json_payload() {
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");
    let payload = json!({"test": "value", "number": 42});

    let package = SlowPackage::new_json_payload(recipient.clone(), sender.clone(), &payload);

    assert_eq!(package.package_type().unwrap(), PackageType::Json);
    assert_eq!(package.recipient_id(), &recipient);
    assert_eq!(package.sender_id(), &sender);
    assert_eq!(package.hop_count(), 0);
    assert_eq!(package.json_payload().unwrap(), payload);
}

#[test]
fn test_packae_new_bin_payload() {
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");
    let binary_data = vec![1, 2, 3, 4, 5];

    let package = SlowPackage::new_bin_payload(recipient.clone(), sender.clone(), &binary_data);

    assert_eq!(package.package_type().unwrap(), PackageType::Bin);
    assert_eq!(package.recipient_id(), &recipient);
    assert_eq!(package.sender_id(), &sender);
    assert_eq!(package.hop_count(), 0);
    assert_eq!(package.payload, binary_data);

    // Test serialization/deserialization of binary payload package
    let serialized = package.pack(package.package_id());
    let deserialized =
        SlowPackage::unpack(&serialized).expect("Failed to deserialize binary package");

    // Verify the deserialized package matches the original
    assert_eq!(deserialized.package_type().unwrap(), PackageType::Bin);
    assert_eq!(deserialized.recipient_id(), &recipient);
    assert_eq!(deserialized.sender_id(), &sender);
    assert_eq!(deserialized.hop_count(), 0);
    assert_eq!(deserialized.payload, binary_data);
}

#[test]
fn test_package_ping_pong() {
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");

    // Test ping package
    let ping = SlowPackage::new_ping(recipient.clone(), sender.clone());
    assert_eq!(ping.package_type().unwrap(), PackageType::Ping);
    assert_eq!(ping.recipient_id(), &recipient);
    assert_eq!(ping.sender_id(), &sender);
    assert_eq!(ping.hop_count(), 0);
    assert!(ping.payload.is_empty());

    // Test serialization/deserialization of ping package
    let serialized_ping = ping.pack(ping.package_id());
    let deserialized_ping =
        SlowPackage::unpack(&serialized_ping).expect("Failed to deserialize ping package");
    assert_eq!(deserialized_ping.package_type().unwrap(), PackageType::Ping);
    assert_eq!(deserialized_ping.recipient_id(), &recipient);
    assert_eq!(deserialized_ping.sender_id(), &sender);
    assert_eq!(deserialized_ping.hop_count(), 0);
    assert!(deserialized_ping.payload.is_empty());

    // Test pong package
    let pong = SlowPackage::new_pong(recipient.clone(), sender.clone());
    assert_eq!(pong.package_type().unwrap(), PackageType::Pong);
    assert_eq!(pong.recipient_id(), &recipient);
    assert_eq!(pong.sender_id(), &sender);
    assert_eq!(pong.hop_count(), 0);
    assert!(pong.payload.is_empty());

    // Test serialization/deserialization of pong package
    let serialized_pong = pong.pack(pong.package_id());
    let deserialized_pong =
        SlowPackage::unpack(&serialized_pong).expect("Failed to deserialize pong package");
    assert_eq!(deserialized_pong.package_type().unwrap(), PackageType::Pong);
    assert_eq!(deserialized_pong.recipient_id(), &recipient);
    assert_eq!(deserialized_pong.sender_id(), &sender);
    assert_eq!(deserialized_pong.hop_count(), 0);
    assert!(deserialized_pong.payload.is_empty());
}

#[test]
fn test_package_hello() {
    let sender = JunctionId::new("sender");
    let package_id = 12345;

    let hello = SlowPackage::new_hello(package_id, sender.clone());

    assert_eq!(hello.package_type().unwrap(), PackageType::Hello);
    assert_eq!(hello.sender_id(), &sender);
    assert_eq!(hello.package_id(), package_id);
    assert_eq!(hello.hop_count(), 0);
    assert!(hello.payload.is_empty());
}

#[test]
fn test_package_serialization_deserialization() {
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");
    let payload = json!({"key": "value"});

    let original = SlowPackage::new_json_payload(recipient.clone(), sender.clone(), &payload);

    // Serialize
    let serialized = original.pack(original.package_id());

    // Deserialize
    let deserialized = SlowPackage::unpack(&serialized).expect("Failed to deserialize package");

    // Verify
    assert_eq!(deserialized.package_type().unwrap(), PackageType::Json);
    assert_eq!(deserialized.recipient_id(), &recipient);
    assert_eq!(deserialized.sender_id(), &sender);
    assert_eq!(deserialized.hop_count(), 0);
    assert_eq!(deserialized.json_payload().unwrap(), payload);
}

#[test]
fn test_package_id() {
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");
    let package = SlowPackage::new_ping(recipient, sender);

    assert_eq!(package.package_id(), 0);

    let mut package_with_id = package;
    package_with_id.set_package_id(42);

    assert_eq!(package_with_id.package_id(), 42);
}

#[test]
fn test_package_hops() {
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");
    let mut package = SlowPackage::new_ping(recipient, sender);

    assert_eq!(package.hop_count(), 0);

    let new_hops = package.increment_hops();
    assert_eq!(new_hops, 1);
    assert_eq!(package.hop_count(), 1);

    package.increment_hops();
    assert_eq!(package.hop_count(), 2);
}

#[test]
fn test_package_invalid() {
    // Create an invalid serialized package (by corrupting the data)
    let recipient = JunctionId::new("recipient");
    let sender = JunctionId::new("sender");
    let original = SlowPackage::new_ping(recipient, sender);

    let mut serialized = original.pack(original.package_id());
    // Corrupt the data by truncating it
    serialized.truncate(serialized.len() - 5);

    // Attempt to deserialize
    let result = SlowPackage::unpack(&serialized);
    assert!(result.is_none(), "Should return None for invalid data");
}

#[test]
fn test_package_type_conversion() {
    // Test valid conversions
    assert_eq!(u8::from(PackageType::Hello), 0);
    assert_eq!(u8::from(PackageType::Ping), 1);
    assert_eq!(u8::from(PackageType::Pong), 2);
    assert_eq!(u8::from(PackageType::Json), 3);
    assert_eq!(u8::from(PackageType::Bin), 4);

    assert_eq!(PackageType::try_from(0).unwrap(), PackageType::Hello);
    assert_eq!(PackageType::try_from(1).unwrap(), PackageType::Ping);
    assert_eq!(PackageType::try_from(2).unwrap(), PackageType::Pong);
    assert_eq!(PackageType::try_from(3).unwrap(), PackageType::Json);
    assert_eq!(PackageType::try_from(4).unwrap(), PackageType::Bin);

    // Test invalid conversion
    assert!(PackageType::try_from(5).is_err());
}
