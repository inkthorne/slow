use slow::tcp::tcp_listener::SlowTcpListener;
use slow::tcp::tcp_stream::SlowTcpStream;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::task;

#[tokio::test]
async fn test_tcp_stream() {
    let addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
    let listener = SlowTcpListener::new(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    // Spawn accept task
    let accept_handle = task::spawn(async move {
        let stream = listener.accept().await.unwrap();
        stream
    });

    // Connect client
    let stream1 = SlowTcpStream::connect(server_addr).await.unwrap();

    // Wait for the connection to be accepted
    let stream2 = accept_handle.await.unwrap();

    // Test data exchange
    let test_data = b"Hello from client!";
    stream1.write(test_data).await.unwrap();

    let mut receive_buffer = [0u8; 32];
    let bytes_read = stream2.read(&mut receive_buffer).await.unwrap();
    assert_eq!(&receive_buffer[..bytes_read], test_data);

    // Test reverse direction
    let response_data = b"Hello from server!";
    stream2.write(response_data).await.unwrap();

    let mut response_buffer = [0u8; 32];
    let bytes_read = stream1.read(&mut response_buffer).await.unwrap();
    assert_eq!(&response_buffer[..bytes_read], response_data);
}
