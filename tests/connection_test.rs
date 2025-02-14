use std::net::SocketAddr;
use slow::connection::SlowConnection;
use slow::datagram::SlowDatagram;

#[tokio::test]
async fn test_connection_pair() {
    let addr1: SocketAddr = "127.0.0.1:5555".parse().unwrap();
    let connection1 = SlowConnection::new(addr1).await.unwrap();

    let addr2: SocketAddr = "127.0.0.1:6666".parse().unwrap();
    let connection2 = SlowConnection::new(addr2).await.unwrap();

    let target_addr: SocketAddr = connection2.local_addr().unwrap();
    let junction_id = 1234;
    let value = serde_json::json!({"key": "value"});
    let datagram = SlowDatagram::new(junction_id, &value).unwrap();
    connection1.send_datagram(&target_addr, &datagram).await.unwrap();

    let (received_datagram, src) = connection2.wait_for_datagram().await.unwrap();
    assert_eq!(received_datagram.get_json().unwrap(), value);
    assert_eq!(src, addr1);
}
