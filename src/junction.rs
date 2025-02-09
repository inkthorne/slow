use crate::connection::{JsonConnection, JsonPacket};
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde_json::Value;

pub struct SlowJunction {
    connection: JsonConnection,
    received_from: HashSet<SocketAddr>,
    send_queue: Arc<Mutex<VecDeque<Value>>>,
    received_queue: Arc<Mutex<VecDeque<JsonPacket>>>,
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
            send_queue: Arc::new(Mutex::new(VecDeque::new())),
            received_queue: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    fn update(&mut self) {
        while let Some(json_packet) = self.connection.recv() {
            self.on_packet_received(json_packet);
        }

        let mut queue = self.send_queue.lock().unwrap();
        while let Some(json) = queue.pop_front() {
            for addr in &self.received_from {
                self.connection.send(&addr.to_string(), &json).expect("Failed to send JSON packet");
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.update();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn dump_addresses(&self) {
        for addr in &self.received_from {
            println!("{}", addr);
        }
    }

    pub fn send(&self, json: Value) {
        let mut queue = self.send_queue.lock().unwrap();
        queue.push_back(json);
    }

    fn on_packet_received(&mut self, json_packet: JsonPacket) {
        println!("Received JSON: {:?}", json_packet.json);
        self.received_from.insert(json_packet.addr);
        {
            let mut queue = self.received_queue.lock().unwrap();
            queue.push_back(json_packet);
        }
        self.dump_addresses();
    }
}
