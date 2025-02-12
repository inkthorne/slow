use crate::connection::{JsonPacket, SlowConnection};
use crate::datagram::SlowDatagram;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub struct SlowJunction {
    connection: SlowConnection,
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
        let connection = SlowConnection::new(addr)?;
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
}

impl SlowJunction {
    /// Updates the state of the `SlowJunction` by processing received packets and sending queued JSON values.
    fn update(&self) {
        while let Some((slow_datagram, sender_addr)) = self.connection.recv() {
            self.on_packet_received(slow_datagram, sender_addr);
        }

        let mut queue = self.send_queue.lock().unwrap();
        while let Some(json) = queue.pop_front() {
            for addr in self.known_junctions.lock().unwrap().iter() {
                self.connection
                    .send(addr, &json)
                    .expect("Failed to send JSON packet");
            }
        }
    }

    /// Runs the main loop of the `SlowJunction`, periodically calling `update`.
    fn run(&self) {
        loop {
            self.update();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Handles a received datagram by forwarding it and updating the known junctions and received queue.
    ///
    /// # Arguments
    ///
    /// * `slow_datagram` - A `SlowDatagram` that was received.
    fn on_packet_received(&self, mut slow_datagram: SlowDatagram, sender_addr: SocketAddr) {
        if slow_datagram.decrement_hops() {
            self.forward(&slow_datagram, sender_addr);
        }
        if let Some(json) = slow_datagram.get_json() {
            let json_packet = JsonPacket {
                addr: sender_addr,
                json,
            };
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

    /// Forwards a `SlowDatagram` to all peers except the sender.
    ///
    /// # Arguments
    ///
    /// * `datagram` - A reference to a `SlowDatagram` to be forwarded.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    fn forward(&self, datagram: &SlowDatagram, sender_addr: SocketAddr) {
        let known_junctions = self.known_junctions.lock().unwrap();
        for addr in known_junctions.iter() {
            if *addr != sender_addr {
                self.connection
                    .send_datagram(addr, datagram)
                    .expect("Failed to forward datagram");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn create_test_junction() -> Arc<SlowJunction> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        SlowJunction::new(addr).expect("Failed to create test junction")
    }

    #[test]
    fn test_new_junction() {
        let junction = create_test_junction();
        assert_eq!(junction.known_junctions.lock().unwrap().len(), 0);
        assert_eq!(junction.send_queue.lock().unwrap().len(), 0);
        assert_eq!(junction.received_queue.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_send() {
        let junction = create_test_junction();
        let json = serde_json::json!({"key": "value"});
        junction.send(json.clone());
        assert_eq!(junction.send_queue.lock().unwrap().len(), 1);
        assert_eq!(junction.send_queue.lock().unwrap().pop_front().unwrap(), json);
    }

    #[test]
    fn test_recv() {
        let junction = create_test_junction();
        let json_packet = JsonPacket {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345),
            json: serde_json::json!({"key": "value"}),
        };
        junction.received_queue.lock().unwrap().push_back(json_packet.clone());
        assert_eq!(junction.recv().unwrap(), json_packet);
    }

    #[test]
    fn test_seed() {
        let junction = create_test_junction();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345);
        junction.seed(addr);
        assert!(junction.known_junctions.lock().unwrap().contains(&addr));
    }

    #[test]
    fn test_get_address() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let junction = SlowJunction::new(addr).expect("Failed to create test junction");
        assert_eq!(junction.get_address(), addr);
    }
}
