use super::tcp_listener::SlowTcpListener;
use super::tcp_stream::SlowTcpStream;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

const HELLO_MESSAGE: &[u8] = b"SLOW_HELLO";
const HELLO_RESPONSE: &[u8] = b"SLOW_WELCOME";

pub struct SlowTcpLink {
    stream: SlowTcpStream,
}

impl SlowTcpLink {
    pub async fn connect(addr: SocketAddr) -> io::Result<Self> {
        let stream = SlowTcpStream::connect(addr).await?;
        let slow_link = Self { stream };
        if !slow_link.hello().await {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "Hello handshake failed",
            ));
        }

        println!("connect success!");
        Ok(slow_link)
    }

    pub async fn listen(addr: SocketAddr) -> io::Result<Self> {
        let listener = SlowTcpListener::new(addr).await?;
        let stream = listener.accept().await?;
        let slow_link = Self { stream };
        if !slow_link.welcome().await {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "Welcome handshake failed",
            ));
        }

        println!("listen success!");
        Ok(slow_link)
    }
}

impl SlowTcpLink {
    pub async fn send(&self, data: &[u8]) -> io::Result<usize> {
        self.stream.send(data).await
    }

    pub async fn receive(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream.receive(buffer).await
    }

    pub async fn hello(&self) -> bool {
        // Send the hello message
        let send_result = self.send(HELLO_MESSAGE).await;
        if send_result.is_err() {
            return false;
        }

        // Wait for response with 5 second timeout
        let mut buffer = [0u8; 32];
        let receive_result = timeout(Duration::from_secs(5), self.receive(&mut buffer)).await;

        match receive_result {
            Ok(Ok(bytes_read)) => {
                // Check if the response matches what we expect
                bytes_read == HELLO_RESPONSE.len() && &buffer[..bytes_read] == HELLO_RESPONSE
            }
            _ => false,
        }
    }

    pub async fn welcome(&self) -> bool {
        // Read and verify the hello message with 5 second timeout
        let mut buffer = [0u8; 32];
        let receive_result = timeout(Duration::from_secs(5), self.receive(&mut buffer)).await;
        let bytes_read = match receive_result {
            Ok(Ok(n)) => n,
            _ => return false,
        };

        if bytes_read != HELLO_MESSAGE.len() || &buffer[..bytes_read] != HELLO_MESSAGE {
            return false;
        }

        // Send the welcome response
        if let Err(_) = self.send(HELLO_RESPONSE).await {
            return false;
        }

        true
    }
}
