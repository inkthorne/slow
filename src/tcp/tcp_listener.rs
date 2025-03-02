use super::tcp_stream::SlowTcpStream;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct SlowTcpListener {
    listener: TcpListener,
}

impl SlowTcpListener {
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self { listener })
    }

    pub async fn accept(&self) -> std::io::Result<SlowTcpStream> {
        let (tokio_stream, _addr) = self.listener.accept().await?;
        let slow_stream = SlowTcpStream::new(tokio_stream);
        Ok(slow_stream)
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}
