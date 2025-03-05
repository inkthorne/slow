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
    /// The read half of the underlying Tokio TCP stream
    reader: Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
    /// The write half of the underlying Tokio TCP stream
    writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
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
        // Split the stream into read and write halves
        let (read_half, write_half) = stream.into_split();

        SlowTcpStream {
            reader: Arc::new(Mutex::new(read_half)),
            writer: Arc::new(Mutex::new(write_half)),
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
        let mut write_half = self.writer.lock().await;
        let data_len = data.len();

        // Use write_all to ensure all bytes are written
        write_half.write_all(data).await?;

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
        let mut read_half = self.reader.lock().await;
        read_half.read(buf).await
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
        let mut read_half = self.reader.lock().await;
        read_half.read_exact(buf).await?;
        Ok(buf.len())
    }

    /// Get a clone of the read half for sharing
    ///
    /// # Returns
    /// * `Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>` - A clone of the read half reference
    pub fn clone_read_half(&self) -> Arc<Mutex<tokio::net::tcp::OwnedReadHalf>> {
        Arc::clone(&self.reader)
    }

    /// Get a clone of the write half for sharing
    ///
    /// # Returns
    /// * `Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>` - A clone of the write half reference
    pub fn clone_write_half(&self) -> Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>> {
        Arc::clone(&self.writer)
    }

    /// Get a clone of the inner stream halves for sharing
    ///
    /// This method is maintained for backward compatibility
    ///
    /// # Returns
    /// * `Arc<Mutex<TcpStream>>` - Not available anymore, will panic
    pub fn clone_inner(&self) -> Arc<Mutex<TcpStream>> {
        panic!(
            "clone_inner is no longer available since the stream is now split into read and write halves"
        )
    }
}
