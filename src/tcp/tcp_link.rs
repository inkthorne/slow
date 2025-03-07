use super::tcp_frame::SlowTcpFrame;
use super::tcp_listener::SlowTcpListener;
use super::tcp_stream::SlowTcpStream;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::timeout;

const HELLO_MESSAGE: &[u8] = b"SLOW_HELLO";
const HELLO_RESPONSE: &[u8] = b"SLOW_WELCOME";

// Static counter for assigning unique IDs to each SlowTcpLink
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// A TCP-based link for the SLOW protocol that handles connection establishment
/// and data transfer with length-prefixed framing.
pub struct SlowTcpLink {
    /// The underlying TCP stream for this link
    stream: SlowTcpStream,
    /// Unique identifier for this link instance
    id: u64,
}

// ---
// SlowTcpLink: Associated Functions
// ---

impl SlowTcpLink {
    /// Creates a new SlowTcpLink with the given stream.
    ///
    /// This function handles assigning a unique ID to the link.
    ///
    /// # Arguments
    /// * `stream` - The TCP stream for this link
    ///
    /// # Returns
    /// A new SlowTcpLink instance
    fn new(stream: SlowTcpStream) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Self { stream, id }
    }

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
        let slow_link = Self::new(stream);
        if !slow_link.hello().await {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "Hello handshake failed",
            ));
        }
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
        let slow_link = Self::new(stream);
        if !slow_link.welcome().await {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "Welcome handshake failed",
            ));
        }
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
    /// Returns the unique identifier of this link
    ///
    /// # Returns
    /// The numeric ID assigned to this link when it was created
    pub fn id(&self) -> u64 {
        self.id
    }

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
