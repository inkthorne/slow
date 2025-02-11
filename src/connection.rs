use crate::datagram::SlowDatagram;
use serde_json::Value;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};

pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

pub struct SlowConnection {
    socket: UdpSocket,
}

impl SlowConnection {
    /// Creates a new `SlowConnection` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` that specifies the address to bind to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowConnection` or an error.
    pub fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(SlowConnection { socket })
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

    /// Sends a `SlowDatagram` to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - A reference to the `SocketAddr` of the recipient.
    /// * `datagram` - A reference to the `SlowDatagram` to send.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - A result indicating success or an error.
    pub fn send_datagram(&self, addr: &SocketAddr, datagram: &SlowDatagram) -> std::io::Result<()> {
        let packaged_data = datagram.package();
        self.socket.send_to(&packaged_data, addr)?;
        Ok(())
    }

    /// Receives a datagram from the socket.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowDatagram, SocketAddr)>` - An option containing the received datagram and the source address or `None` if no datagram is available.
    pub fn recv(&self) -> Option<(SlowDatagram, SocketAddr)> {
        let mut buf = [0; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let datagram = &buf[..amt];
                SlowDatagram::unpackage(datagram).map(|d| (d, src))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::net::SocketAddr;

    #[test]
    fn test_send_and_recv() {
        let addr1: SocketAddr = "127.0.0.1:1111".parse().unwrap();
        let connection1 = SlowConnection::new(addr1);
        assert!(connection1.is_ok());
        let connection1 = connection1.unwrap();

        let addr2: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let connection2 = SlowConnection::new(addr2);
        assert!(connection2.is_ok());
        let connection2 = connection2.unwrap();

        let target_addr: SocketAddr = connection2.socket.local_addr().unwrap();
        let junction_id = 1234;
        let value = json!({"key": "value"});
        let datagram = SlowDatagram::new(junction_id, &value).unwrap();
        let send_result = connection1.send_datagram(&target_addr, &datagram);
        assert!(send_result.is_ok());

        let received = connection2.recv();
        assert!(received.is_some());
    }
}
