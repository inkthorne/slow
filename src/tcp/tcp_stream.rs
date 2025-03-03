use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Represents a TCP stream in the slow network stack
///
/// This struct provides a thread-safe wrapper around a Tokio TcpStream,
/// allowing multiple parts of the application to share and use the same
/// network connection safely.
pub struct SlowTcpStream {
    /// The underlying Tokio TCP stream
    inner: Arc<Mutex<TcpStream>>,
}

impl SlowTcpStream {
    /// Connects to a remote address and returns a new SlowTcpStream
    ///
    /// # Arguments
    /// * `addr` - The socket address to connect to
    ///
    /// # Returns
    /// * `io::Result<Self>` - A new SlowTcpStream or an IO error if connection fails
    pub async fn connect(addr: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self::new(stream))
    }

    /// Creates a new SlowTcpStream from an existing TcpStream
    ///
    /// # Arguments
    /// * `stream` - An existing Tokio TcpStream to wrap
    ///
    /// # Returns
    /// * `Self` - A new SlowTcpStream instance
    pub fn new(stream: TcpStream) -> Self {
        SlowTcpStream {
            inner: Arc::new(Mutex::new(stream)),
        }
    }

    /// Sends data over the TCP stream
    ///
    /// This method ensures that all data is written to the stream.
    ///
    /// # Arguments
    /// * `data` - The byte slice containing the data to send
    ///
    /// # Returns
    /// * `io::Result<usize>` - The number of bytes written or an IO error
    pub async fn write(&self, data: &[u8]) -> io::Result<usize> {
        let mut stream = self.inner.lock().await;
        let data_len = data.len();

        // Use write_all to ensure all bytes are written
        stream.write_all(data).await?;

        Ok(data_len)
    }

    /// Receives data from the TCP stream
    ///
    /// This method will read available data into the provided buffer.
    ///
    /// # Arguments
    /// * `buf` - The mutable byte slice to read data into
    ///
    /// # Returns
    /// * `io::Result<usize>` - The number of bytes read or an IO error
    pub async fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut stream = self.inner.lock().await;
        stream.read(buf).await
    }

    /// Reads exactly enough bytes to fill the buffer
    ///
    /// This method will read until the buffer is filled completely or an error occurs.
    /// If an error occurs before filling the buffer, an error is returned.
    /// If EOF is encountered before filling the buffer, it will return an UnexpectedEof error.
    ///
    /// # Arguments
    /// * `buf` - The mutable byte slice to read data into
    ///
    /// # Returns
    /// * `io::Result<usize>` - The number of bytes read or an IO error
    pub async fn read_exact(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut stream = self.inner.lock().await;
        stream.read_exact(buf).await
    }

    /// Get a clone of the inner stream for sharing
    ///
    /// This method provides access to the underlying Arc<Mutex<TcpStream>>,
    /// allowing the stream to be shared across multiple consumers.
    ///
    /// # Returns
    /// * `Arc<Mutex<TcpStream>>` - A clone of the inner TCP stream reference
    pub fn clone_inner(&self) -> Arc<Mutex<TcpStream>> {
        Arc::clone(&self.inner)
    }
}
