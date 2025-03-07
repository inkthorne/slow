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

    // Test sending maximum size data
    let max_size = SlowTcpLink::max_frame_size();
    println!("Testing maximum size data transfer ({}B)...", max_size);

    // Create a buffer filled with a repeating pattern for testing
    let max_test_data = vec![0xAB; max_size];

    // Create a receive buffer of the same size
    let mut max_receive_buffer = vec![0u8; max_size];

    // Send the maximum size data
    connector_link
        .send(&max_test_data)
        .await
        .expect("Failed to send maximum size data");

    // Receive the maximum size data
    let max_bytes_read = listener_link
        .receive(&mut max_receive_buffer)
        .await
        .expect("Failed to receive maximum size data");

    // Verify the data was received correctly
    assert_eq!(
        max_bytes_read, max_size,
        "Received data size does not match max frame size"
    );
    assert_eq!(
        &max_receive_buffer[..max_bytes_read],
        &max_test_data,
        "Maximum size received data does not match sent data"
    );

    // Test sending data that exceeds the maximum frame size by 1 byte
    println!("Testing oversized data transfer ({}B)...", max_size + 1);

    // Create a buffer that's 1 byte larger than the maximum allowed size
    let oversized_data = vec![0xCD; max_size + 1];

    // Attempt to send the oversized data - this should fail
    let send_result = connector_link.send(&oversized_data).await;

    // Verify that the send operation failed with the expected error
    assert!(send_result.is_err(), "Sending oversized data should fail");

    if let Err(e) = send_result {
        // Check that it's the right kind of error
        assert_eq!(
            e.kind(),
            std::io::ErrorKind::InvalidInput,
            "Expected InvalidInput error kind for oversized data"
        );

        // Verify the error message mentions the size limit
        let error_msg = e.to_string().to_lowercase();
        assert!(
            error_msg.contains("exceeds") && error_msg.contains("limit"),
            "Error message should indicate that the data exceeds the size limit"
        );
    }

    // Test receiving after closing the connector link
    println!("Testing reception after connector closure...");

    // Close the connector side of the connection
    connector_link
        .close()
        .await
        .expect("Failed to close connector link");

    // Wait a bit to ensure the close propagates
    sleep(Duration::from_millis(100)).await;

    // Try to receive data on the listener side - should fail due to closed connection
    let mut buffer = [0u8; 64];
    let receive_result = listener_link.receive(&mut buffer).await;

    // Verify that receiving fails with the expected error
    assert!(
        receive_result.is_err(),
        "Receiving after connection closed should fail"
    );

    if let Err(e) = receive_result {
        // Usually TCP disconnections result in EOF (UnexpectedEof) or BrokenPipe errors
        println!("Received expected error after connection closed: {:?}", e);
        assert!(
            matches!(
                e.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionAborted
            ),
            "Expected connection-related error, got: {:?}",
            e.kind()
        );
    }
}
