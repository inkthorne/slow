use crate::connection::{JsonConnection, JsonPacket};
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde_json::Value;

pub struct SlowJunction {
    connection: JsonConnection,
    known_junctions: Arc<Mutex<HashSet<SocketAddr>>>,
    send_queue: Arc<Mutex<VecDeque<Value>>>,
    received_queue: Arc<Mutex<VecDeque<JsonPacket>>>,
    addr: SocketAddr, // Add addr field
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
    pub fn new(addr: SocketAddr) -> std::io::Result<Arc<Self>> {
        let connection = JsonConnection::new(addr)?;
        let junction = Arc::new(Self {
            connection,
            known_junctions: Arc::new(Mutex::new(HashSet::new())),
            send_queue: Arc::new(Mutex::new(VecDeque::new())),
            received_queue: Arc::new(Mutex::new(VecDeque::new())),
            addr, // Initialize addr field
        });

        let junction_clone = Arc::clone(&junction);
        std::thread::spawn(move || {
            junction_clone.run();
        });

        Ok(junction)
    }

    /// Prints the addresses of all peers that have sent packets to the `SlowJunction`.
    pub fn print_known_junctions(&self) {
        for addr in self.known_junctions.lock().unwrap().iter() {
            println!("{}", addr);
        }
    }

    /// Queues a JSON value to be sent to all peers.
    ///
    /// # Arguments
    ///
    /// * `json` - A `Value` representing the JSON data to be queued.
    pub fn send(&self, json: Value) {
        let mut queue = self.send_queue.lock().unwrap();
        queue.push_back(json);
    }

    /// Receives a JSON packet from the received queue.
    ///
    /// # Returns
    ///
    /// * `Option<JsonPacket>` - An optional JSON packet if available.
    pub fn recv(&self) -> Option<JsonPacket> {
        let mut queue = self.received_queue.lock().unwrap();
        queue.pop_front()
    }

    /// Adds a seed address to the set of received addresses.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` to be added to the set of received addresses.
    pub fn seed(&self, addr: SocketAddr) {
        let mut known_junctions = self.known_junctions.lock().unwrap();
        known_junctions.insert(addr);
    }

    /// Returns the `SocketAddr` of the `SlowJunction`.
    pub fn get_address(&self) -> SocketAddr {
        self.addr
    }

    /// Forwards a JSON packet to all peers except the sender.
    ///
    /// # Arguments
    ///
    /// * `packet` - A reference to a `JsonPacket` to be forwarded.
    pub fn forward(&self, packet: &JsonPacket) {
        let sender_addr = packet.addr;
        let known_junctions = self.known_junctions.lock().unwrap();
        for addr in known_junctions.iter() {
            if *addr != sender_addr {
                self.connection.send(&addr.to_string(), &packet.json).expect("Failed to forward JSON packet");
            }
        }
    }
}

impl SlowJunction {
    fn update(&self) {
        while let Some(json_packet) = self.connection.recv() {
            self.on_packet_received(json_packet);
        }

        let mut queue = self.send_queue.lock().unwrap();
        while let Some(json) = queue.pop_front() {
            for addr in self.known_junctions.lock().unwrap().iter() {
                self.connection.send(&addr.to_string(), &json).expect("Failed to send JSON packet");
            }
        }
    }

    fn run(&self) {
        loop {
            self.update();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn on_packet_received(&self, json_packet: JsonPacket) {
        self.forward(&json_packet);
        {
            let mut known_junctions = self.known_junctions.lock().unwrap();
            known_junctions.insert(json_packet.addr);
        }
        {
            let mut queue = self.received_queue.lock().unwrap();
            queue.push_back(json_packet);
        }
    }
}
