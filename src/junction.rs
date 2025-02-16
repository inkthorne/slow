use crate::connection::{JsonPacket, SlowConnection};
use crate::datagram::SlowDatagram;
use crate::route::RouteTable;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

#[derive(Clone, Hash, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JunctionId {
    id: String,
}

impl JunctionId {
    pub fn new(id: &str) -> Self {
        JunctionId { id: id.to_string() }
    }
}

impl std::fmt::Display for JunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// A `SlowJunction` represents a network junction that can send and receive datagrams, manage known junctions,
/// and handle JSON packets.
///
/// This struct provides methods to create a new junction, send and receive JSON packets, manage known junctions,
/// and run the main loop. It is designed to work asynchronously using the Tokio runtime.
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
    junction_id: JunctionId,

    /// A flag to indicate if the thread should terminate.
    terminate: Arc<AtomicBool>,

    /// A notification to signal when a datagram is added to the send queue.
    send_notify: Arc<Notify>,

    /// A notification to signal when a datagram is added to the receive queue.
    receive_notify: Arc<Notify>,

    /// A counter for the number of pong messages received.
    pong_counter: Arc<Mutex<u32>>,

    /// The route table for the junction.
    route_table: Arc<Mutex<RouteTable>>,
}

impl Drop for SlowJunction {
    fn drop(&mut self) {
        self.terminate.store(true, Ordering::SeqCst);
    }
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
    /// * `Result<Arc<Self>, std::io::Error>` - A result containing a new instance of `SlowJunction` or an error.
    pub async fn new(addr: SocketAddr, junction_id: JunctionId) -> std::io::Result<Arc<Self>> {
        let connection = SlowConnection::new(addr).await?;
        let junction = Arc::new(Self {
            connection,
            known_junctions: Arc::new(Mutex::new(HashSet::new())),
            send_queue: Arc::new(Mutex::new(VecDeque::new())),
            received_queue: Arc::new(Mutex::new(VecDeque::new())),
            addr,
            junction_id, // use passed JunctionId directly
            terminate: Arc::new(AtomicBool::new(false)),
            send_notify: Arc::new(Notify::new()),
            receive_notify: Arc::new(Notify::new()),
            pong_counter: Arc::new(Mutex::new(0)),
            route_table: Arc::new(Mutex::new(RouteTable::new())),
        });

        let junction_clone = Arc::clone(&junction);
        tokio::spawn(async move {
            junction_clone.run().await;
        });

        Ok(junction)
    }

    /// Prints the addresses of all known junctions.
    pub async fn print_known_junctions(&self) {
        let known_junctions = self.known_junctions.lock().await;
        for addr in known_junctions.iter() {
            println!("{}", addr);
        }
    }

    /// Queues a JSON value to be sent to all known junctions.
    ///
    /// # Arguments
    ///
    /// * `json` - A `Value` representing the JSON data to be queued.
    /// * `recipient_id` - A &str representing the recipient ID.
    pub async fn send(&self, json: Value, recipient_id: &JunctionId) {
        let mut queue = self.send_queue.lock().await;
        let datagram = SlowDatagram::new(recipient_id.clone(), self.junction_id.clone(), &json)
            .expect("Failed to create datagram");
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

    /// Adds a seed address to the set of known junction addresses.
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

    /// Waits for a notification that there are items in the received queue and returns the JSON packet.
    ///
    /// # Returns
    ///
    /// * `Option<JsonPacket>` - An optional JSON packet if available.
    pub async fn wait_for_datagram(&self) -> Option<JsonPacket> {
        self.receive_notify.notified().await;
        let mut queue = self.received_queue.lock().await;
        queue.pop_front()
    }

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

    /// Updates the known junctions by adding the sender address.
    ///
    /// # Arguments
    ///
    /// * `sender_addr` - The `SocketAddr` of the sender to be added.
    async fn update_route_table(&self, sender_addr: SocketAddr) {
        let mut known_junctions = self.known_junctions.lock().await;
        known_junctions.insert(sender_addr);
    }

    /// Handles a received datagram by forwarding it and updating the known junctions and received queue.
    ///
    /// # Arguments
    ///
    /// * `slow_datagram` - A `SlowDatagram` that was received.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn on_datagram_received(&self, slow_datagram: SlowDatagram, sender_addr: SocketAddr) {
        // Always add sender to known junctions
        self.update_route_table(sender_addr).await;
        self.insert_route(&slow_datagram.get_recipient_id(), sender_addr, 1, 0.0)
            .await;

        if *slow_datagram.get_recipient_id() != self.junction_id {
            self.forward(slow_datagram, sender_addr).await;
            return;
        }
        if let Some(json) = slow_datagram.get_json() {
            if json["type"] == "ping" {
                self.on_ping_received(json).await;
                return;
            }
            if json["type"] == "pong" {
                self.on_pong_received().await;
                return;
            }
            let json_packet = JsonPacket {
                addr: sender_addr,
                json,
            };
            let mut queue = self.received_queue.lock().await;
            queue.push_back(json_packet);
            self.receive_notify.notify_one();
        }
    }

    /// Forwards a `SlowDatagram` to all known junctions except the sender.
    ///
    /// # Arguments
    ///
    /// * `datagram` - A `SlowDatagram` to be forwarded.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn forward(&self, mut datagram: SlowDatagram, sender_addr: SocketAddr) {
        if !datagram.decrement_hops() {
            return;
        }
        self.send_to_known_junctions(datagram, Some(sender_addr))
            .await;
    }

    /// Sends a `SlowDatagram` to all known junctions except the specified sender.
    ///
    /// # Arguments
    ///
    /// * `datagram` - The `SlowDatagram` to be sent.
    /// * `exclude_addr` - The `SocketAddr` of the sender to be excluded.
    pub async fn send_to_known_junctions(
        &self,
        datagram: SlowDatagram,
        exclude_addr: Option<SocketAddr>,
    ) {
        let known_junctions = self.known_junctions.lock().await;
        for addr in known_junctions.iter() {
            if Some(*addr) != exclude_addr {
                self.connection
                    .send_datagram(&datagram, addr)
                    .await
                    .expect("Failed to send datagram");
            }
        }
    }

    /// Waits for a datagram via `connection.recv_datagram()` and returns it.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowDatagram, SocketAddr)>` - An optional tuple containing the received datagram and sender address.
    pub async fn read_datagram(&self) -> Option<(SlowDatagram, SocketAddr)> {
        self.connection.recv_datagram().await
    }

    /// Sends all queued datagrams to known junctions, excluding the address `0.0.0.0:0`.
    async fn pump_send(&self) {
        self.send_notify.notified().await;
        let mut queue = self.send_queue.lock().await;
        while let Some(datagram) = queue.pop_front() {
            self.send_to_known_junctions(datagram, None).await;
        }
    }

    /// Receives a datagram via `connection.recv_datagram()` and processes it.
    async fn pump_recv(&self) {
        if let Some((slow_datagram, sender_addr)) = self.connection.recv_datagram().await {
            self.on_datagram_received(slow_datagram, sender_addr).await;
        }
    }

    /// Sends a pong message to a specific `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A u16 representing the recipient ID.
    pub async fn pong(&self, recipient_id: &JunctionId) {
        let message =
            serde_json::json!({"type": "pong", "sender_id": self.junction_id.to_string()});
        self.send(message, recipient_id).await;
    }

    /// Sends a ping message to a specific `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The target junction id as a &str.
    pub async fn ping(&self, junction_id: &JunctionId) {
        let message =
            serde_json::json!({"type": "ping", "sender_id": self.junction_id.to_string()});
        self.send(message, junction_id).await;
    }

    /// Returns the current value of the pong counter.
    ///
    /// # Returns
    ///
    /// * `u32` - The current value of the pong counter.
    pub async fn get_pong_counter(&self) -> u32 {
        let counter = self.pong_counter.lock().await;
        *counter
    }

    /// Increments the pong counter when a pong message is received.
    async fn on_pong_received(&self) {
        let mut counter = self.pong_counter.lock().await;
        *counter += 1;
    }

    /// Handles a received ping message by sending a pong response.
    ///
    /// # Arguments
    ///
    /// * `json` - The JSON data of the received ping message.
    async fn on_ping_received(&self, json: Value) {
        let sender_str = json["sender_id"].as_str().unwrap();
        let sender_id = JunctionId::new(sender_str);
        self.pong(&sender_id).await;
    }

    /// Inserts a route into the route table.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The `JunctionId` of the junction.
    /// * `addr` - The `SocketAddr` of the junction.
    /// * `hops` - The number of hops to the junction.
    /// * `time` - The time to the junction.
    async fn insert_route(&self, junction_id: &JunctionId, addr: SocketAddr, hops: u16, time: f32) {
        let mut route_table = self.route_table.lock().await;
        route_table.insert_route(junction_id, addr, hops, time);
    }

    /// Gets the best route to a junction.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The `JunctionId` of the junction.
    ///
    /// # Returns
    ///
    /// * `Option<SocketAddr>` - The best route to the junction.
    async fn get_best_route(&self, junction_id: &JunctionId) -> Option<SocketAddr> {
        let route_table = self.route_table.lock().await;
        route_table.get_best_route(junction_id)
    }
}
