use serde_json::Value;
use crate::udp;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct SlowConnection {
    received_from: HashSet<SocketAddr>,
}

impl SlowConnection {
    pub fn new() -> Self {
        Self {
            received_from: HashSet::new(),
        }
    }

    /// Listens for UDP packets on the specified port.
    /// 
    /// # Arguments
    /// 
    /// * `port` - A u16 that specifies the port number to bind to.
    /// * `callback` - A function to be called when a packet is received.
    /// 
    /// This function does not return a value.
    pub fn listen<F>(&mut self, port: u16, callback: F)
    where
        F: Fn(&Value) + Send + 'static,
    {
        let address = format!("localhost:{}", port);
        let socket = std::net::UdpSocket::bind(&address).expect("Couldn't bind to address");
        loop {
            if let Some(packet) = udp::listen_for_slow_packet(&socket) {
                self.received_from.insert(packet.addr);
                callback(&packet.json);
                self.dump_addresses();
            }
        }
    }

    /// Prints the contents of `received_from`.
    pub fn dump_addresses(&self) {
        for addr in &self.received_from {
            println!("{}", addr);
        }
    }
}
