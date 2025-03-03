use super::tcp_listener::SlowTcpListener;
use super::tcp_stream::SlowTcpStream;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

const HELLO_MESSAGE: &[u8] = b"SLOW_HELLO";
const HELLO_RESPONSE: &[u8] = b"SLOW_WELCOME";
const MAX_FRAME_SIZE: usize = 1024 * 1024; // 1MB limit

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
        MAX_FRAME_SIZE
    }
}

// ---
// SlowTcpLink: Public Functions
// ---

impl SlowTcpLink {
    /// Sends data over the link with length-prefix framing.
    ///
    /// The data is wrapped with length prefixes at both the start and end for validation.
    /// There is a 1MB size limit for any single transmission.
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
        // Ensure the data is not too large
        if data.len() > MAX_FRAME_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Data size exceeds 1MB limit: {} bytes", data.len()),
            ));
        }

        let len = data.len() as u32;
        let len_bytes = len.to_be_bytes();

        // Send the length prefix
        self.stream.write(&len_bytes).await?;

        // Send the actual data
        let bytes_sent = self.stream.write(data).await?;

        // Send the length suffix (same as prefix for validation)
        self.stream.write(&len_bytes).await?;

        // Return the number of data bytes sent (not including the length prefix/suffix)
        Ok(bytes_sent)
    }

    /// Receives data from the link with length-prefix framing validation.
    ///
    /// Reads a length prefix, the actual data, and a length suffix,
    /// validating that the prefix and suffix match.
    ///
    /// # Arguments
    /// * `buffer` - Buffer to store the received data
    ///
    /// # Returns
    /// The number of bytes read into the buffer
    ///
    /// # Errors
    /// Returns an error if the buffer is too small, if reading fails,
    /// or if the length prefix and suffix don't match
    pub async fn receive(&self, buffer: &mut [u8]) -> io::Result<usize> {
        // First read the length prefix (4 bytes for u32)
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await?;

        // Convert bytes to u32 (from network byte order)
        let expected_len = u32::from_be_bytes(len_bytes);

        // Ensure the buffer is large enough
        if buffer.len() < expected_len as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "SlowTcpLink: buffer too small: {} bytes needed but only {} bytes available.",
                    expected_len,
                    buffer.len()
                ),
            ));
        }

        // Now read the actual data using read_exact to ensure we get all the expected bytes
        self.stream
            .read_exact(&mut buffer[..expected_len as usize])
            .await?;

        // Read and validate the length suffix
        let mut suffix_len_bytes = [0u8; 4];
        self.stream.read_exact(&mut suffix_len_bytes).await?;
        let suffix_len = u32::from_be_bytes(suffix_len_bytes);

        // Ensure the length prefix matches the length suffix
        if expected_len != suffix_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Length mismatch: prefix {} bytes but suffix indicates {} bytes",
                    expected_len, suffix_len
                ),
            ));
        }

        // Return the number of data bytes read
        Ok(expected_len as usize)
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
