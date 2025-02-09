use serde_json::Value;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};

pub struct JsonConnection {
    socket: UdpSocket,
}

pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

impl JsonConnection {
    /// Creates a new `JsonConnection` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` that specifies the address to bind to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `JsonConnection` or an error.
    pub fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(JsonConnection { socket })
    }

    pub fn send(&self, addr: &str, json: &Value) -> std::io::Result<()> {
        let data = serde_json::to_vec(json)?;
        self.socket.send_to(&data, addr)?;
        Ok(())
    }

    pub fn recv(&self) -> Option<JsonPacket> {
        let mut buf = [0; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let data = &buf[..amt];
                match serde_json::from_slice::<Value>(data) {
                    Ok(json) => Some(JsonPacket { addr: src, json }),
                    Err(_) => None,
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }
}
