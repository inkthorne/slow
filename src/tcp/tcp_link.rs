use super::tcp_frame::SlowTcpFrame;
use super::tcp_listener::SlowTcpListener;
use super::tcp_stream::SlowTcpStream;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

const HELLO_MESSAGE: &[u8] = b"SLOW_HELLO";
const HELLO_RESPONSE: &[u8] = b"SLOW_WELCOME";

/// A TCP-based link for the SLOW protocol that handles connection establishment
/// and data transfer with length-prefixed framing.
pub struct SlowTcpLink {
    stream: SlowTcpStream,
}

// ---
// SlowTcpLink: Associated Functions
// ---

impl SlowTcpLink {
    /// Connects to a remote SLOW endpoint and performs a handshake.
    ///
    /// # Arguments
    /// * `addr` - The socket address to connect to
    ///
    /// # Returns
    /// A new SlowTcpLink if connection and handshake succeed
    ///
    /// # Errors
    /// Returns an error if connection fails or handshake is unsuccessful
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

    /// Listens for an incoming SLOW connection and performs a welcome handshake.
    ///
    /// # Arguments
    /// * `addr` - The socket address to listen on
    ///
    /// # Returns
    /// A new SlowTcpLink if connection and handshake succeed
    ///
    /// # Errors
    /// Returns an error if listening fails or handshake is unsuccessful
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

    /// Returns the maximum allowed frame size for this link implementation.
    ///
    /// # Returns
    /// The maximum number of bytes that can be sent in a single frame
    pub fn max_frame_size() -> usize {
        SlowTcpFrame::max_frame_size()
    }
}

// ---
// SlowTcpLink: Public Functions
// ---

impl SlowTcpLink {
    /// Sends data over the link with length-prefix framing.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send
    ///
    /// # Returns
    /// The number of bytes sent (not including framing)
    ///
    /// # Errors
    /// Returns an error if the data is too large or if the transmission fails
    pub async fn send(&self, data: &[u8]) -> io::Result<usize> {
        SlowTcpFrame::send(data, &self.stream).await
    }

    /// Receives data from the link with length-prefix framing validation.
    ///
    /// # Arguments
    /// * `buffer` - Buffer to store the received data
    ///
    /// # Returns
    /// The number of bytes read into the buffer
    ///
    /// # Errors
    /// Returns an error if the buffer is too small, if reading fails,
    /// or if the frame is invalid.
    pub async fn receive(&self, buffer: &mut [u8]) -> io::Result<usize> {
        SlowTcpFrame::receive(buffer, &self.stream).await
    }

    /// Closes the TCP connection.
    ///
    /// This method will shut down the underlying TCP stream,
    /// preventing further communication on this link.
    ///
    /// # Returns
    /// * `io::Result<()>` - Ok if the shutdown was successful, or an IO error
    pub async fn close(&self) -> io::Result<()> {
        self.stream.close().await
    }
}

// ---
// SlowTcpLink: Prviate Functions
// ---

impl SlowTcpLink {
    /// Performs the client side of the handshake by sending a hello message
    /// and verifying the response.
    ///
    /// # Returns
    /// `true` if the handshake was successful, `false` otherwise
    async fn hello(&self) -> bool {
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

    /// Performs the server side of the handshake by receiving a hello message
    /// and sending back a welcome response.
    ///
    /// # Returns
    /// `true` if the handshake was successful, `false` otherwise
    async fn welcome(&self) -> bool {
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
