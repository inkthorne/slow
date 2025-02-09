use crate::json_connection::{JsonConnection, JsonPacket};
use serde_json::Value;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct SlowConnection {
    received_from: HashSet<SocketAddr>,
    connection: JsonConnection, // Added connection as a member
}

impl SlowConnection {
    /// Creates a new `SlowConnection` instance.
    ///
    /// # Arguments
    ///
    /// * `port` - A u16 that specifies the port number to bind to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowConnection` or an error.
    pub fn new(port: u16) -> std::io::Result<Self> {
        Ok(Self {
            received_from: HashSet::new(),
            connection: JsonConnection::new(port)?, // Initialize connection
        })
    }

    /// Listens for UDP packets on the specified port.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function to be called when a packet is received.
    ///
    /// This function does not return a value.
    pub fn listen<F>(&mut self, callback: F)
    where
        F: Fn(&Value) + Send + 'static,
    {
        loop {
            match self.connection.recv() {
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
