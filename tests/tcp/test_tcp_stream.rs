use slow::tcp::tcp_listener::SlowTcpListener;
use slow::tcp::tcp_stream::SlowTcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::task;

#[tokio::test]
async fn test_tcp_socket() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
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
    stream1.send(test_data).await.unwrap();

    let mut receive_buffer = [0u8; 32];
    let bytes_read = stream2.receive(&mut receive_buffer).await.unwrap();
    assert_eq!(&receive_buffer[..bytes_read], test_data);

    // Test reverse direction
    let response_data = b"Hello from server!";
    stream2.send(response_data).await.unwrap();

    let mut response_buffer = [0u8; 32];
    let bytes_read = stream1.receive(&mut response_buffer).await.unwrap();
    assert_eq!(&response_buffer[..bytes_read], response_data);
}
