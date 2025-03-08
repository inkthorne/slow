use serde_json::json;
use slow::junction::JunctionId;
use slow::junction::SlowJunction;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

#[tokio::test]
async fn test_junction_line() {
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1110);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2220);
    let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3330);
    let addr4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4440);

    let junction1 = SlowJunction::new(addr1, JunctionId::new("1"))
        .await
        .expect("Failed to create junction1");
    let junction2 = SlowJunction::new(addr2, JunctionId::new("2"))
        .await
        .expect("Failed to create junction2");
    let junction3 = SlowJunction::new(addr3, JunctionId::new("3"))
        .await
        .expect("Failed to create junction3");
    let junction4 = SlowJunction::new(addr4, JunctionId::new("4"))
        .await
        .expect("Failed to create junction4");

    junction1.join(addr2).await;
    junction2.join(addr3).await;
    junction3.join(addr4).await;

    // Wait for hellos to finish.
    tokio::time::sleep(Duration::from_millis(250)).await;

    let ping = json!({"key": "ping"});
    junction1
        .send(ping.clone(), junction4.get_junction_id())
        .await;

    // Delay before receiving the package
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Check waiting package count before receiving
    assert_eq!(junction4.get_waiting_package_count().await, 1);

    // Check if the package was received by junction4
    let received_package = junction4.recv().await.unwrap();
    assert_eq!(received_package.json, ping);
    assert_eq!(received_package.addr, addr3);

    // Send pong response back to junction1
    let pong = json!({"key": "pong"});
    junction4.send(pong.clone(), &JunctionId::new("1")).await;

    // Wait for pong to arrive
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Check waiting package count before receiving pong
    assert_eq!(junction1.get_waiting_package_count().await, 1);

    // Verify junction1 received the pong
    let pong_package = junction1.recv().await.unwrap();
    assert_eq!(pong_package.json, pong);
    assert_eq!(pong_package.addr, addr2);
}

#[tokio::test]
async fn test_junction_square() {
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1111);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2221);
    let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3331);
    let addr4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4441);

    let junction_id1 = JunctionId::new("1");
    let junction_id2 = JunctionId::new("2");
    let junction_id3 = JunctionId::new("3");
    let junction_id4 = JunctionId::new("4");

    let junction1 = SlowJunction::new(addr1, junction_id1.clone())
        .await
        .expect("Failed to create junction1");
    let junction2 = SlowJunction::new(addr2, junction_id2.clone())
        .await
        .expect("Failed to create junction2");
    let junction3 = SlowJunction::new(addr3, junction_id3.clone())
        .await
        .expect("Failed to create junction3");
    let junction4 = SlowJunction::new(addr4, junction_id4.clone())
        .await
        .expect("Failed to create junction4");

    // Create square topology: junction1 -> (junction2, junction3) -> junction4
    junction1.join(addr2).await;
    junction1.join(addr3).await;
    junction2.join(addr4).await;
    junction3.join(addr4).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    let ping = json!({"key": "ping"});
    junction1.send(ping.clone(), &junction_id4).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    assert_eq!(junction4.get_waiting_package_count().await, 1); // Should receive 2 packages but only accept 1
    assert_eq!(junction4.get_duplicate_package_count(), 1); // Should receive 1 duplicate packages

    let received_package = junction4.recv().await.unwrap();
    assert_eq!(received_package.json, ping);
    assert!(received_package.addr == addr2 || received_package.addr == addr3);

    let pong = json!({"key": "pong"});
    junction4.send(pong.clone(), &junction_id1).await;

    tokio::time::sleep(Duration::from_millis(250)).await;

    assert_eq!(junction1.get_waiting_package_count().await, 1); // Should receive from onlye best path

    let pong_package = junction1.recv().await.unwrap();
    assert_eq!(pong_package.json, pong);
    assert!(pong_package.addr == addr2 || pong_package.addr == addr3);

    // Assert best route to junction "1" from "4" exists
    assert!(junction4.get_best_route(&junction_id1).await.is_some());
}

#[tokio::test]
async fn test_junction_pyramid() {
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1112);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2222);
    let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3332);
    let addr4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4442);
    let addr5 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5552);

    let junction_id1 = JunctionId::new("1");
    let junction_id2 = JunctionId::new("2");
    let junction_id3 = JunctionId::new("3");
    let junction_id4 = JunctionId::new("4");
    let junction_id5 = JunctionId::new("5");

    let junction1 = SlowJunction::new(addr1, junction_id1.clone())
        .await
        .expect("Failed to create junction1");
    let junction2 = SlowJunction::new(addr2, junction_id2.clone())
        .await
        .expect("Failed to create junction2");
    let junction3 = SlowJunction::new(addr3, junction_id3.clone())
        .await
        .expect("Failed to create junction3");
    let junction4 = SlowJunction::new(addr4, junction_id4.clone())
        .await
        .expect("Failed to create junction4");
    let junction5 = SlowJunction::new(addr5, junction_id5.clone())
        .await
        .expect("Failed to create junction5");

    // Create pyramid topology: junction1 -> (junction2, junction3) -> junction4
    junction1.join(addr2).await;
    junction1.join(addr3).await;
    junction2.join(addr4).await;
    junction3.join(addr4).await;

    // Join all junctions with junction5
    junction1.join(addr5).await;
    junction2.join(addr5).await;
    junction3.join(addr5).await;
    junction4.join(addr5).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    let ping = json!({"key": "ping"});
    junction1.send(ping.clone(), &junction_id4).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    assert_eq!(junction5.get_duplicate_package_count(), 2); // Should have received 2 duplicate packages from junction2 & 3
    assert_eq!(junction4.get_waiting_package_count().await, 1); // Should receive from all paths but only accept 1
    assert_eq!(junction4.get_duplicate_package_count(), 2); // Should have received 2 duplicate packages

    let received_package = junction4.recv().await.unwrap();
    assert_eq!(received_package.json, ping);

    // Assert best route to junction "1" from "4" exists
    assert!(junction4.get_best_route(&junction_id1).await.is_some());

    let pong = json!({"key": "pong"});
    junction4.send(pong.clone(), &junction_id1).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    assert_eq!(junction1.get_waiting_package_count().await, 1); // Should receive only from one best path
}

#[tokio::test]
async fn test_junction_pair() {
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5555);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6666);

    let junction1 = SlowJunction::new(addr1, JunctionId::new("1"))
        .await
        .expect("Failed to create junction1");
    let junction2 = SlowJunction::new(addr2, JunctionId::new("2"))
        .await
        .expect("Failed to create junction2");

    junction1.join(addr2).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    let ping = json!({"key": "ping"});
    junction1.send(ping.clone(), &JunctionId::new("2")).await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(junction2.get_waiting_package_count().await, 1);

    let received_package = junction2.recv().await.unwrap();
    assert_eq!(received_package.json, ping);
    assert_eq!(received_package.addr, addr1);

    // Send pong response back to junction1
    let pong = json!({"key": "pong"});
    junction2.send(pong.clone(), &JunctionId::new("1")).await;

    // Wait for pong to arrive
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Check waiting package count before receiving pong
    assert_eq!(junction1.get_waiting_package_count().await, 1);

    // Verify junction1 received the pong
    let pong_package = junction1.recv().await.unwrap();
    assert_eq!(pong_package.json, pong);
    assert_eq!(pong_package.addr, addr2);
}

#[tokio::test]
async fn test_junction_ping() {
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7777);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8888);

    let junction1 = SlowJunction::new(addr1, JunctionId::new("1"))
        .await
        .expect("Failed to create junction1");
    let junction2 = SlowJunction::new(addr2, JunctionId::new("2"))
        .await
        .expect("Failed to create junction2");

    junction1.join(addr2).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Use the ping function
    junction1.ping(junction2.get_junction_id()).await;

    // Wait for pong to arrive
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Assert the pong count of junction1 is 1
    assert_eq!(junction1.get_pong_counter().await, 1);
}

#[test]
fn test_junction_id_serialization() {
    // Create a JunctionId
    let original_id = JunctionId::new("test-junction-123");

    // Pack it into bytes
    let packed = original_id.pack();

    // Unpack bytes back to JunctionId
    let unpacked_id = JunctionId::unpack(&packed).expect("Failed to unpack JunctionId");

    // Verify the unpacked ID matches the original
    assert_eq!(unpacked_id, original_id);

    // Test with a different ID
    let another_id = JunctionId::new("another-junction-456");
    let another_packed = another_id.pack();
    let another_unpacked =
        JunctionId::unpack(&another_packed).expect("Failed to unpack JunctionId");
    assert_eq!(another_unpacked, another_id);

    // Test that they don't equal each other
    assert_ne!(original_id, another_id);

    // Test error cases
    assert!(
        JunctionId::unpack(&[]).is_none(),
        "Should return None for empty data"
    );
    assert!(
        JunctionId::unpack(&[5, 0]).is_none(),
        "Should return None for insufficient data"
    );

    // Test with invalid UTF-8
    let mut invalid_utf8 = vec![3, 0]; // length 3
    invalid_utf8.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Invalid UTF-8 bytes
    assert!(
        JunctionId::unpack(&invalid_utf8).is_none(),
        "Should return None for invalid UTF-8"
    );
}
