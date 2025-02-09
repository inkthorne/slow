use crate::connection::JsonConnection;
use std::collections::HashSet;
use std::net::SocketAddr;

pub struct SlowJunction {
    connection: JsonConnection,
    received_from: HashSet<SocketAddr>,
}

impl SlowJunction {
    /// Creates a new `SlowJunction` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` that specifies the address to bind to.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowJunction` or an error.
    pub fn new(addr: SocketAddr) -> std::io::Result<Self> {
        let connection = JsonConnection::new(addr)?;
        Ok(Self {
            connection,
            received_from: HashSet::new(),
        })
    }

    pub fn update<F>(&mut self, callback: F)
    where
        F: Fn(&serde_json::Value),
    {
        while let Some(json_packet) = self.connection.recv() {
            self.received_from.insert(json_packet.addr);
            callback(&json_packet.json);
            self.dump_addresses();
        }
    }

    pub fn run<F>(&mut self, callback: F)
    where
        F: Fn(&serde_json::Value) + Send + 'static,
    {
        loop {
            self.update(&callback);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn dump_addresses(&self) {
        for addr in &self.received_from {
            println!("{}", addr);
        }
    }
}
