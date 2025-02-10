use crate::datagram::SlowDatagram;
use serde_json::Value;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};

pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

pub struct JsonConnection {
    socket: UdpSocket,
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

    /// Sends a JSON value to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - A reference to the `SocketAddr` of the recipient.
    /// * `json` - A reference to the JSON value to send.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error.
    pub fn send(&self, addr: &SocketAddr, json: &Value) -> std::io::Result<()> {
        let datagram = SlowDatagram::new(addr.port(), json).unwrap();
        let packaged_data = datagram.package();
        self.socket.send_to(&packaged_data, addr)?;
        Ok(())
    }

    /// Receives a JSON packet from the socket.
    ///
    /// # Returns
    ///
    /// * `Option<JsonPacket>` - An option containing the received JSON packet or `None` if no packet is available.
    pub fn recv(&self) -> Option<JsonPacket> {
        let mut buf = [0; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let datagram = &buf[..amt];
                if let Some(slow_datagram) = SlowDatagram::unpackage(datagram) {
                    if let Some(json) = slow_datagram.get_json() {
                        Some(JsonPacket { addr: src, json })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }
}
