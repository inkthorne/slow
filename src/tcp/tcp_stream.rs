use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Represents a TCP stream in the slow network stack
pub struct SlowTcpStream {
    /// The underlying Tokio TCP stream
    inner: Arc<Mutex<TcpStream>>,
}

impl SlowTcpStream {
    /// Connects to a remote address and returns a new SlowTcpStream
    pub async fn connect(addr: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self::new(stream))
    }

    /// Creates a new SlowTcpStream from an existing TcpStream
    pub fn new(stream: TcpStream) -> Self {
        SlowTcpStream {
            inner: Arc::new(Mutex::new(stream)),
        }
    }

    /// Sends data over the TCP stream
    pub async fn send(&self, data: &[u8]) -> io::Result<usize> {
        let mut stream = self.inner.lock().await;
        stream.write(data).await
    }

    /// Receives data from the TCP stream
    pub async fn receive(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut stream = self.inner.lock().await;
        stream.read(buf).await
    }

    /// Get a clone of the inner stream for sharing
    pub fn clone_inner(&self) -> Arc<Mutex<TcpStream>> {
        Arc::clone(&self.inner)
    }
}
