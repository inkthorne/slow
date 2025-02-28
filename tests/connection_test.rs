use slow::connection::SlowSocket;
use slow::junction::JunctionId;
use slow::package::SlowPackage;
use std::net::SocketAddr;

#[tokio::test]
async fn test_connection_pair() {
    let addr1: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:8082".parse().unwrap();
    let connection1 = SlowSocket::new(addr1).await.unwrap();
    let connection2 = SlowSocket::new(addr2).await.unwrap();

    let json = serde_json::json!({ "key": "value" });
    let sender_id = JunctionId::new("A");
    let recipient_id = JunctionId::new("B");
    let package = SlowPackage::new_json_payload(recipient_id, sender_id, &json);

    // Send package from connection1 to connection2
    connection1.send_package(&package, &addr2).await.unwrap();

    // Receive package on connection2
    let received = connection2.receive_package().await.unwrap();

    assert_eq!(received.1, addr1);
    assert_eq!(received.0.json_payload().unwrap(), json);
}
