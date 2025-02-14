use crate::connection::{JsonPacket, SlowConnection};
use crate::datagram::SlowDatagram;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

pub struct SlowJunction {
    /// The connection used by the junction.
    connection: SlowConnection,

    /// A set of known junction addresses.
    known_junctions: Arc<Mutex<HashSet<SocketAddr>>>,

    /// A queue of datagrams to be sent.
    send_queue: Arc<Mutex<VecDeque<SlowDatagram>>>,

    /// A queue of received JSON packets.
    received_queue: Arc<Mutex<VecDeque<JsonPacket>>>,

    /// The address of the junction.
    addr: SocketAddr,

    /// The recipient ID for the junction.
    recipient_id: u16,

    /// A flag to indicate if the thread should terminate.
    terminate: Arc<AtomicBool>,

    /// A notification to signal when a datagram is added to the send queue.
    send_notify: Arc<Notify>,

    /// A notification to signal when a datagram is added to the receive queue.
    receive_notify: Arc<Notify>,
}

impl SlowJunction {
    /// Creates a new `SlowJunction` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` that specifies the address to bind to.
    /// * `recipient_id` - A `u16` representing the recipient ID.
    ///
    /// # Returns
    ///
    /// * `Result<Self, std::io::Error>` - A result containing a new instance of `SlowJunction` or an error.
    pub async fn new(addr: SocketAddr, recipient_id: u16) -> std::io::Result<Arc<Self>> {
        let connection = SlowConnection::new(addr).await?;
        let junction = Arc::new(Self {
            connection,
            known_junctions: Arc::new(Mutex::new(HashSet::new())),
            send_queue: Arc::new(Mutex::new(VecDeque::new())),
            received_queue: Arc::new(Mutex::new(VecDeque::new())),
            addr,
            recipient_id,
            terminate: Arc::new(AtomicBool::new(false)),
            send_notify: Arc::new(Notify::new()),
            receive_notify: Arc::new(Notify::new()),
        });

        let junction_clone = Arc::clone(&junction);
        tokio::spawn(async move {
            junction_clone.run().await;
        });

        Ok(junction)
    }

    /// Prints the addresses of all peers that have sent packets to the `SlowJunction`.
    pub async fn print_known_junctions(&self) {
        let known_junctions = self.known_junctions.lock().await;
        for addr in known_junctions.iter() {
            println!("{}", addr);
        }
    }

    /// Queues a JSON value to be sent to all peers.
    ///
    /// # Arguments
    ///
    /// * `json` - A `Value` representing the JSON data to be queued.
    /// * `recipient_id` - A `u16` representing the recipient ID.
    pub async fn send(&self, json: Value, recipient_id: u16) {
        let mut queue = self.send_queue.lock().await;
        let datagram = SlowDatagram::new(recipient_id, &json).expect("Failed to create datagram");
        queue.push_back(datagram);
        self.send_notify.notify_one();
    }

    /// Receives a JSON packet from the received queue.
    ///
    /// # Returns
    ///
    /// * `Option<JsonPacket>` - An optional JSON packet if available.
    pub async fn recv(&self) -> Option<JsonPacket> {
        let mut queue = self.received_queue.lock().await;
        queue.pop_front()
    }

    /// Adds a seed address to the set of received addresses.
    ///
    /// # Arguments
    ///
    /// * `addr` - A `SocketAddr` to be added to the set of known junction addresses.
    pub async fn seed(&self, addr: SocketAddr) {
        let mut known_junctions = self.known_junctions.lock().await;
        known_junctions.insert(addr);
    }

    /// Returns the `SocketAddr` of the `SlowJunction`.
    pub fn get_address(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the number of packets waiting to be received.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of packets in the received queue.
    pub async fn waiting_packet_count(&self) -> usize {
        let queue = self.received_queue.lock().await;
        queue.len()
    }

    /// Waits for a notification that there are items in the received_queue and returns the datagram.
    pub async fn wait_for_datagram(&self) -> Option<JsonPacket> {
        self.receive_notify.notified().await;
        let mut queue = self.received_queue.lock().await;
        queue.pop_front()
    }
}

impl Drop for SlowJunction {
    fn drop(&mut self) {
        self.terminate.store(true, Ordering::SeqCst);
    }
}

impl SlowJunction {
    /// Updates the state of the `SlowJunction` by processing received packets and sending queued JSON values.
    async fn update2(&self) {
        tokio::select! {
            _= self.pump_send() => {}
            _ = self.pump_recv() => {}
        }
    }

    /// Runs the main loop of the `SlowJunction`, periodically calling `update2`.
    async fn run(&self) {
        while !self.terminate.load(Ordering::SeqCst) {
            self.update2().await;
        }
    }

    /// Handles a received datagram by forwarding it and updating the known junctions and received queue.
    ///
    /// # Arguments
    ///
    /// * `slow_datagram` - A `SlowDatagram` that was received.
    async fn on_datagram_received(&self, slow_datagram: SlowDatagram, sender_addr: SocketAddr) {
        // Always add sender to known junctions
        {
            let mut known_junctions = self.known_junctions.lock().await;
            known_junctions.insert(sender_addr);
        }

        if slow_datagram.get_recipient_id() != self.recipient_id {
            self.forward(slow_datagram, sender_addr).await;
            return;
        }
        if let Some(json) = slow_datagram.get_json() {
            let json_packet = JsonPacket {
                addr: sender_addr,
                json,
            };
            let mut queue = self.received_queue.lock().await;
            queue.push_back(json_packet);
            self.receive_notify.notify_one();
        }
    }

    /// Forwards a `SlowDatagram` to all peers except the sender.
    ///
    /// # Arguments
    ///
    /// * `datagram` - A `SlowDatagram` to be forwarded.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn forward(&self, mut datagram: SlowDatagram, sender_addr: SocketAddr) {
        if !datagram.decrement_hops() {
            return;
        }
        let known_junctions = self.known_junctions.lock().await;
        for addr in known_junctions.iter() {
            if *addr != sender_addr {
                self.connection
                    .send_datagram(addr, &datagram)
                    .await
                    .expect("Failed to forward datagram");
            }
        }
    }

    /// Waits for a datagram via connection.recv() and returns it.
    pub async fn read_datagram(&self) -> Option<(SlowDatagram, SocketAddr)> {
        self.connection.recv().await
    }

    async fn pump_send(&self) {
        self.send_notify.notified().await;
        let mut queue = self.send_queue.lock().await;
        while let Some(datagram) = queue.pop_front() {
            for addr in self.known_junctions.lock().await.iter() {
                self.connection
                    .send_datagram(addr, &datagram)
                    .await
                    .expect("Failed to send datagram");
            }
        }
    }

    async fn pump_recv(&self) {
        if let Some((slow_datagram, sender_addr)) = self.connection.recv().await {
            self.on_datagram_received(slow_datagram, sender_addr).await;
        }
    }
}
