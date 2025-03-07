use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::Notify;

/// Represents a TCP stream in the slow network stack
///
/// This struct provides a thread-safe wrapper around a Tokio TcpStream,
/// allowing multiple parts of the application to share and use the same
/// network connection safely.
pub struct SlowTcpStream {
    /// The read half of the underlying Tokio TCP stream
    reader: Mutex<OwnedReadHalf>,
    /// The write half of the underlying Tokio TCP stream
    writer: Mutex<OwnedWriteHalf>,
    /// Used to notify the read() functions to return EOF
    close_notify: Notify,
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
        let (reader, writer) = stream.into_split();

        SlowTcpStream {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
            close_notify: Notify::new(),
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
        let mut writer = self.writer.lock().await;
        let data_len = data.len();

        // Use write_all to ensure all bytes are written
        writer.write_all(data).await?;

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
        let mut reader = self.reader.lock().await;
        select! {
            result = reader.read(buf) => {
                result
            }
            _ = self.close_notify.notified() => {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
            }
        }
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
        let mut reader = self.reader.lock().await;
        select! {
            result = reader.read_exact(buf) => {
                result
            }
            _ = self.close_notify.notified() => {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
            }
        }
    }

    /// Closes the TCP connection
    ///
    /// This method will shut down both the read and write halves of the connection,
    /// preventing further communication on this stream.
    ///
    /// # Returns
    /// * `io::Result<()>` - Ok if the shutdown was successful, or an IO error
    pub async fn close(&self) -> io::Result<()> {
        self.close_notify.notify_waiters();

        // Acquire locks for both reader & writer
        let _reader = self.reader.lock().await;
        let mut writer = self.writer.lock().await;

        // Shutdown the write half first (sends FIN packet)
        writer.shutdown().await?;
        Ok(())
    }
}
