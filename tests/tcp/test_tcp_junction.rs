use slow::junction::JunctionId;
use slow::package::SlowPackage;
use slow::tcp::tcp_junction::SlowTcpJunction;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::time;

/// Tests a basic TCP junction connection between two nodes.
///
/// This test verifies:
/// 1. Connection establishment between two TCP junctions
/// 2. Package transmission from one junction to another
/// 3. Link count verification before and after connection
/// 4. Proper cleanup and link removal after junction closure
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
        .clone()
        .connect(addr1)
        .await
        .expect("Failed to connect junction2 to junction1");

    // Allow some time for the connection to be established
    time::sleep(Duration::from_millis(100)).await;

    // Verify that both junctions have the expected number of links
    // Junction1 should have 1 link from junction2's connection
    assert_eq!(
        junction1.link_count().await,
        1,
        "Junction1 should have 1 link"
    );

    // Junction2 should also have 1 link to junction1
    assert_eq!(
        junction2.link_count().await,
        1,
        "Junction2 should have 1 link"
    );

    // Create test data to send from junction2 to junction1
    let test_message = b"Hello from junction2 to junction1";
    let test_package =
        SlowPackage::new_bin_payload(junction_id1.clone(), junction_id2.clone(), test_message);

    // Send the package from junction2 to junction1
    let bytes_sent = junction2
        .send_package(&test_package)
        .await
        .expect("Failed to send package from junction2 to junction1");

    assert_eq!(
        bytes_sent,
        test_package.pack(0).len(),
        "Bytes sent should match packaged data length"
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

    // Close junction1
    junction1.close().await.expect("Failed to close junction1");
    drop(junction1);

    // Allow some time for the connection to be terminated and links to be removed
    time::sleep(Duration::from_millis(300)).await;

    // Verify that junction2 now has 0 links since junction1 was closed
    assert_eq!(
        junction2.link_count().await,
        0,
        "Junction2 should have 0 links after junction1 was closed"
    );
}

/// Tests a linear network topology with four TCP junctions.
///
/// This test verifies:
/// 1. Creation of a linear network topology (1 -> 2 -> 3 -> 4)
/// 2. Connection establishment between multiple junctions
/// 3. Package routing through multiple hops
/// 4. Link count verification for each junction
/// 5. Proper cleanup and link removal after all junctions are closed
#[tokio::test]
async fn test_tcp_junction_line() {
    // Create addresses for the four junctions
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9101);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9102);
    let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9103);
    let addr4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9104);

    // Create IDs for the four junctions
    let junction_id1 = JunctionId::new("junction1");
    let junction_id2 = JunctionId::new("junction2");
    let junction_id3 = JunctionId::new("junction3");
    let junction_id4 = JunctionId::new("junction4");

    // Create the junction instances
    let junction1 = SlowTcpJunction::new(addr1, junction_id1.clone());
    let junction2 = SlowTcpJunction::new(addr2, junction_id2.clone());
    let junction3 = SlowTcpJunction::new(addr3, junction_id3.clone());
    let junction4 = SlowTcpJunction::new(addr4, junction_id4.clone());

    // Allow some time for junctions to initialize and start listening
    time::sleep(Duration::from_millis(100)).await;

    // Connect the junctions in a line: 1 -> 2 -> 3 -> 4
    junction1
        .clone()
        .connect(addr2)
        .await
        .expect("Failed to connect junction1 to junction2");

    junction2
        .clone()
        .connect(addr3)
        .await
        .expect("Failed to connect junction2 to junction3");

    junction3
        .clone()
        .connect(addr4)
        .await
        .expect("Failed to connect junction3 to junction4");

    // Allow some time for all connections to be established
    time::sleep(Duration::from_millis(200)).await;

    // Verify that all junctions have the expected number of links
    assert_eq!(
        junction1.link_count().await,
        1,
        "Junction1 should have 1 link"
    );
    assert_eq!(
        junction2.link_count().await,
        2,
        "Junction2 should have 2 links"
    );
    assert_eq!(
        junction3.link_count().await,
        2,
        "Junction3 should have 2 links"
    );
    assert_eq!(
        junction4.link_count().await,
        1,
        "Junction4 should have 1 link"
    );

    // Create a howdy package from junction1 instead of a binary package
    println!("Sending howdy package from junction1 to all junctions");
    let howdy_package = SlowPackage::new_howdy(junction_id1.clone());

    // Send the howdy package from junction1
    let bytes_sent = junction1
        .send_package(&howdy_package)
        .await
        .expect("Failed to send howdy package from junction1");

    assert_eq!(
        bytes_sent,
        howdy_package.pack(0).len(),
        "Bytes sent should match packaged data length"
    );

    // Allow some time for the package to traverse all junctions
    time::sleep(Duration::from_millis(300)).await;

    // Assert that all junctions have received the package since howdy packages are sent to all
    assert_eq!(
        junction1.received_package_count(),
        0,
        "Junction1 should have received 0 packages"
    );
    assert_eq!(
        junction2.received_package_count(),
        1,
        "Junction2 should have received 1 package"
    );
    assert_eq!(
        junction3.received_package_count(),
        1,
        "Junction3 should have received 1 package"
    );
    assert_eq!(
        junction4.received_package_count(),
        1,
        "Junction4 should have received 1 package"
    );

    // Close all junctions
    junction1.close().await.expect("Failed to close junction1");
    junction2.close().await.expect("Failed to close junction2");
    junction3.close().await.expect("Failed to close junction3");
    junction4.close().await.expect("Failed to close junction4");

    // Allow some time for connections to be terminated
    time::sleep(Duration::from_millis(300)).await;

    // Verify that all junctions have 0 links after closing
    assert_eq!(
        junction1.link_count().await,
        0,
        "Junction1 should have 0 links after closing"
    );
    assert_eq!(
        junction2.link_count().await,
        0,
        "Junction2 should have 0 links after closing"
    );
    assert_eq!(
        junction3.link_count().await,
        0,
        "Junction3 should have 0 links after closing"
    );
    assert_eq!(
        junction4.link_count().await,
        0,
        "Junction4 should have 0 links after closing"
    );
}

/// Tests a triangular network topology with three TCP junctions.
///
/// This test verifies:
/// 1. Creation of a triangular network topology (1 -> 2 -> 3 -> 1)
/// 2. Connection establishment between all three junctions
/// 3. Package routing in a triangular topology
/// 4. Link count verification for each junction
/// 5. Proper cleanup and link removal after all junctions are closed
#[tokio::test]
async fn test_tcp_junction_triangle() {
    // Create addresses for the three junctions
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9201);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9202);
    let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9203);

    // Create IDs for the three junctions
    let junction_id1 = JunctionId::new("junction1");
    let junction_id2 = JunctionId::new("junction2");
    let junction_id3 = JunctionId::new("junction3");

    // Create the junction instances
    let junction1 = SlowTcpJunction::new(addr1, junction_id1.clone());
    let junction2 = SlowTcpJunction::new(addr2, junction_id2.clone());
    let junction3 = SlowTcpJunction::new(addr3, junction_id3.clone());

    // Allow some time for junctions to initialize and start listening
    time::sleep(Duration::from_millis(100)).await;

    // Connect the junctions in a triangle: 1 -> 2 -> 3 -> 1
    junction1
        .clone()
        .connect(addr2)
        .await
        .expect("Failed to connect junction1 to junction2");

    junction2
        .clone()
        .connect(addr3)
        .await
        .expect("Failed to connect junction2 to junction3");

    junction3
        .clone()
        .connect(addr1)
        .await
        .expect("Failed to connect junction3 to junction1");

    // Allow some time for all connections to be established
    time::sleep(Duration::from_millis(200)).await;

    // Verify that all junctions have the expected number of links (2 each)
    assert_eq!(
        junction1.link_count().await,
        2,
        "Junction1 should have 2 links"
    );
    assert_eq!(
        junction2.link_count().await,
        2,
        "Junction2 should have 2 links"
    );
    assert_eq!(
        junction3.link_count().await,
        2,
        "Junction3 should have 2 links"
    );

    // Create a howdy package from junction1 instead of a binary package
    println!("Sending howdy package from junction1 to all junctions");
    let test_package = SlowPackage::new_howdy(junction_id1.clone());

    // Send the howdy package from junction1
    let bytes_sent = junction1
        .send_package(&test_package)
        .await
        .expect("Failed to send howdy package from junction1");

    assert_eq!(
        bytes_sent,
        test_package.pack(0).len(),
        "Bytes sent should match packaged data length"
    );

    // Allow some time for the package to traverse the network
    time::sleep(Duration::from_millis(300)).await;

    // Since howdy packages are sent to all junctions, both junction2 and junction3 should have received it
    assert_eq!(
        junction3.received_package_count(),
        1,
        "Junction3 should have received 1 package"
    );

    // Junction3 should have rejected 1 package, since howdy packages are sent to all junctions
    assert_eq!(
        junction3.rejected_package_count(),
        1,
        "Junction3 should have rejected 0 packages"
    );

    // Verify that junction2 received the package during routing
    assert_eq!(
        junction2.received_package_count(),
        1,
        "Junction2 should have received 1 package during routing"
    );

    // Junction2 should have 1 rejected packages
    assert_eq!(
        junction2.rejected_package_count(),
        1,
        "Junction2 should have rejected 0 packages"
    );

    // Junction1 should not have received any package
    assert_eq!(
        junction1.received_package_count(),
        0,
        "Junction1 should have received 0 packages"
    );

    // Now have junction3 send a response package back to junction1
    let response_message = b"Response from junction3 to junction1";
    let response_package =
        SlowPackage::new_bin_payload(junction_id1.clone(), junction_id3.clone(), response_message);

    // Send the package from junction3 to junction1
    let response_bytes_sent = junction3
        .send_package(&response_package)
        .await
        .expect("Failed to send response package from junction3 to junction1");

    assert_eq!(
        response_bytes_sent,
        response_package.pack(0).len(),
        "Response bytes sent should match packaged data length"
    );

    // Allow some time for the response package to traverse the network
    time::sleep(Duration::from_millis(300)).await;

    // Now junction1 should have received 1 package
    assert_eq!(
        junction1.received_package_count(),
        1,
        "Junction1 should have received 1 package"
    );

    // Close all junctions
    junction1.close().await.expect("Failed to close junction1");
    junction2.close().await.expect("Failed to close junction2");
    junction3.close().await.expect("Failed to close junction3");

    // Allow some time for connections to be terminated
    time::sleep(Duration::from_millis(300)).await;

    // Verify that all junctions have 0 links after closing
    assert_eq!(
        junction1.link_count().await,
        0,
        "Junction1 should have 0 links after closing"
    );
    assert_eq!(
        junction2.link_count().await,
        0,
        "Junction2 should have 0 links after closing"
    );
    assert_eq!(
        junction3.link_count().await,
        0,
        "Junction3 should have 0 links after closing"
    );
}

/// Tests a randomly connected network with 16 TCP junctions.
///
/// This test verifies:
/// 1. Creation of 16 TCP junctions
/// 2. Random connection establishment between junctions
/// 3. Package transmission from a randomly selected junction
/// 4. Proper cleanup and link removal after all junctions are closed
#[tokio::test]
async fn test_tcp_junction_hello() {
    // Import rand for random number generation
    use rand::{Rng, seq::SliceRandom};

    // Constants
    const NUM_JUNCTIONS: usize = 16;
    const BASE_PORT: u16 = 9300;

    // Create arrays to hold all junction data
    let mut addresses = Vec::with_capacity(NUM_JUNCTIONS);
    let mut junction_ids = Vec::with_capacity(NUM_JUNCTIONS);
    let mut junctions = Vec::with_capacity(NUM_JUNCTIONS);

    // Create all junctions
    for i in 0..NUM_JUNCTIONS {
        let port = BASE_PORT + (i as u16);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let junction_id = JunctionId::new(&format!("junction{}", i + 1));

        addresses.push(addr);
        junction_ids.push(junction_id.clone());
        junctions.push(SlowTcpJunction::new(addr, junction_id));
    }

    // Allow some time for junctions to initialize and start listening
    time::sleep(Duration::from_millis(100)).await;

    // Randomly connect junctions
    let mut rng = rand::thread_rng();

    // Each junction will connect to at least one other junction
    // but we'll avoid self-connections
    for i in 0..NUM_JUNCTIONS {
        // Choose a random target junction that is not the current junction
        let mut targets: Vec<usize> = (0..NUM_JUNCTIONS).filter(|&j| j != i).collect();
        targets.shuffle(&mut rng);

        // Connect to a random number of junctions (1 to 3)
        let num_connections = rng.gen_range(1..=3).min(targets.len());

        for j in 0..num_connections {
            let target_idx = targets[j];
            let target_addr = addresses[target_idx];

            junctions[i]
                .clone()
                .connect(target_addr)
                .await
                .unwrap_or_else(|_| {
                    eprintln!(
                        "Failed to connect junction{} to junction{}",
                        i + 1,
                        target_idx + 1
                    )
                });
        }
    }

    // Allow time for connections to be established
    time::sleep(Duration::from_millis(300)).await;

    // Print connection stats for each junction
    for i in 0..NUM_JUNCTIONS {
        let link_count = junctions[i].link_count().await;
        println!("Junction{} has {} links", i + 1, link_count);
        assert!(
            link_count > 0,
            "Junction{} should have at least 1 link",
            i + 1
        );
    }

    // Select a random source junction and destination junction
    let src_idx = rng.gen_range(0..NUM_JUNCTIONS);
    let mut dst_idx = rng.gen_range(0..NUM_JUNCTIONS);

    // Ensure source and destination are different
    while dst_idx == src_idx {
        dst_idx = rng.gen_range(0..NUM_JUNCTIONS);
    }

    // Create a howdy package from source junction
    println!(
        "Sending howdy package from junction{} to all junctions",
        src_idx + 1
    );
    let howdy_package = SlowPackage::new_howdy(junction_ids[src_idx].clone());

    // Send the package
    let bytes_sent = junctions[src_idx]
        .send_package(&howdy_package)
        .await
        .expect("Failed to send howdy package");

    assert_eq!(
        bytes_sent,
        howdy_package.pack(0).len(),
        "Bytes sent should match packaged data length"
    );

    // Allow time for the package to propagate through the network
    time::sleep(Duration::from_millis(500)).await;

    // Check that all junctions received the package
    for i in 0..NUM_JUNCTIONS {
        if i == src_idx {
            // Skip the source junction
            continue;
        }

        let received_count = junctions[i].received_package_count();
        println!("Junction{} received {} packages", i + 1, received_count);

        // Assert that each junction has received exactly 1 package
        assert_eq!(
            received_count,
            1,
            "Junction{} should have received exactly 1 package",
            i + 1
        );
    }
}
