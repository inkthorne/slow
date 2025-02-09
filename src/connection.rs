use crate::json_connection::{JsonConnection, JsonPacket};
use serde_json::Value;
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
        let connection = JsonConnection::new(port).expect("Couldn't bind to address");
        loop {
            match connection.recv() {
                Some(JsonPacket { addr, json }) => {
                    self.received_from.insert(addr);
                    callback(&json);
                    self.dump_addresses();
                }
                None => {
                    // No packet received, continue listening
                    continue;
                }
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
