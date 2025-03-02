use slow::tcp::tcp_link::SlowTcpLink;
use std::net::SocketAddr;
use tokio;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_tcp_link() {
    // Use a local address with an ephemeral port
    let addr = "127.0.0.1:12345".parse::<SocketAddr>().unwrap();

    // Start the SlowTcpLink listener in a task
    let listener_handle = tokio::spawn(async move { SlowTcpLink::listen(addr).await });
    sleep(Duration::from_millis(100)).await;

    // Connect to the listener
    let connector_handle = tokio::spawn(async move { SlowTcpLink::connect(addr).await });

    // Wait for both operations to complete
    let (listener_result, connector_result) = tokio::join!(listener_handle, connector_handle);

    // Assert both operations succeeded
    assert!(listener_result.unwrap().is_ok(), "Listener failed");
    assert!(connector_result.unwrap().is_ok(), "Connector failed");
}
