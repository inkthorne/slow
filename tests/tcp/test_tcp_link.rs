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

    // Extract the link instances and make sure they were created successfully
    let listener_link = listener_result.unwrap().expect("Listener failed");
    let connector_link = connector_result.unwrap().expect("Connector failed");

    // Test data transmission from connector to listener
    let test_message = b"Hello from connector!";
    connector_link
        .send(test_message)
        .await
        .expect("Failed to send data from connector");

    let mut receive_buffer = [0u8; 64];
    let bytes_read = listener_link
        .receive(&mut receive_buffer)
        .await
        .expect("Failed to receive data at listener");

    assert_eq!(
        &receive_buffer[..bytes_read],
        test_message,
        "Received data does not match sent data"
    );

    // Test data transmission in the opposite direction (listener to connector)
    let response_message = b"Hello from listener!";
    listener_link
        .send(response_message)
        .await
        .expect("Failed to send data from listener");

    let mut response_buffer = [0u8; 64];
    let response_bytes_read = connector_link
        .receive(&mut response_buffer)
        .await
        .expect("Failed to receive data at connector");

    assert_eq!(
        &response_buffer[..response_bytes_read],
        response_message,
        "Response data does not match sent data"
    );
}
