use slow::junction::JunctionId;
use slow::tcp::tcp_junction::SlowTcpJunction;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn test_tcp_junction() {
    // Create addresses for the two junctions
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9001);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9002);

    // Create IDs for the two junctions
    let junction_id1 = JunctionId::new("junction1");
    let junction_id2 = JunctionId::new("junction2");

    // Create the junction instances
    let junction1 = SlowTcpJunction::new(addr1, junction_id1.clone());
    let junction2 = SlowTcpJunction::new(addr2, junction_id2.clone());

    // Allow some time for junctions to initialize and start listening
    time::sleep(Duration::from_millis(100)).await;

    // Connect junction2 to junction1
    junction2
        .connect(addr1)
        .await
        .expect("Failed to connect junction2 to junction1");

    // Allow some time for the connection to be established
    time::sleep(Duration::from_millis(100)).await;

    // Verify that both junctions have the expected number of links
    // Junction1 should have 1 link from junction2's connection
    assert_eq!(junction1.link_count(), 1, "Junction1 should have 1 link");

    // Junction2 should also have 1 link to junction1
    assert_eq!(junction2.link_count(), 1, "Junction2 should have 1 link");

    // Create test data to send from junction2 to junction1
    let test_data = b"Hello from junction2 to junction1";

    // Send data from junction2 to junction1
    let bytes_sent = junction2
        .send(test_data, &junction_id1)
        .await
        .expect("Failed to send data from junction2 to junction1");

    assert_eq!(
        bytes_sent,
        test_data.len(),
        "Bytes sent should match test data length"
    );

    // Allow some time for the data to be processed
    time::sleep(Duration::from_millis(100)).await;

    // Assert that junction1 has received 1 package and junction2 has received 0 packages
    assert_eq!(
        junction1.received_package_count(),
        1,
        "Junction1 should have received 1 package"
    );
    assert_eq!(
        junction2.received_package_count(),
        0,
        "Junction2 should have received 0 packages"
    );
}
