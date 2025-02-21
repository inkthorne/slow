use crate::connection::SlowConnection;
use crate::datagram::SlowPackage;
use crate::route::RouteTable;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

#[derive(Clone, PartialEq, Debug)]
pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

/// A `JunctionId` represents the unique identifier for a network junction.
///
/// This struct provides methods to create a new junction ID and format it for display.
#[derive(Clone, Hash, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JunctionId {
    /// The unique identifier for the junction.
    id: String,
}

impl JunctionId {
    /// Creates a new `JunctionId` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice that holds the ID.
    ///
    /// # Returns
    ///
    /// * `Self` - A new instance of `JunctionId`.
    pub fn new(id: &str) -> Self {
        JunctionId { id: id.to_string() }
    }
}

impl std::fmt::Display for JunctionId {
    /// Formats the `JunctionId` for display.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a `std::fmt::Formatter`.
    ///
    /// # Returns
    ///
    /// * `std::fmt::Result` - The result of the formatting operation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// A `SlowJunction` represents a network junction that can send and receive packages, manage known junctions,
/// and handle JSON packets.
///
/// This struct provides methods to create a new junction, send and receive JSON packets, manage known junctions,
/// and run the main loop. It is designed to work asynchronously using the Tokio runtime.
pub struct SlowJunction {
    /// The connection used by the junction.
    connection: SlowConnection,

    /// A set of known junction addresses.
    known_junctions: Arc<Mutex<HashSet<SocketAddr>>>,

    /// A queue of packages to be sent.
    send_queue: Arc<Mutex<VecDeque<SlowPackage>>>,

    /// A queue of received JSON packets.
    received_queue: Arc<Mutex<VecDeque<JsonPacket>>>,

    /// The address of the junction.
    addr: SocketAddr,

    /// The recipient ID for the junction.
    junction_id: JunctionId,

    /// A flag to indicate if the thread should terminate.
    terminate: Arc<AtomicBool>,

    /// A notification to signal when a package is added to the send queue.
    send_notify: Arc<Notify>,

    /// A notification to signal when a package is added to the receive queue.
    receive_notify: Arc<Notify>,

    /// A counter for the number of pong messages received.
    pong_counter: Arc<Mutex<u32>>,

    /// The route table for the junction.
    route_table: Arc<Mutex<RouteTable>>,

    /// A counter for the number of packages sent.
    sent_package_count: AtomicU32,

    /// A counter for the number of duplicate packages rejected.
    duplicate_package_count: AtomicUsize,
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
            sent_package_count: AtomicU32::new(0),
            duplicate_package_count: AtomicUsize::new(0),
        });

        let junction_clone = Arc::clone(&junction);
        tokio::spawn(async move {
            junction_clone.run().await;
        });

        Ok(junction)
    }

    /// Logs a message to the console.
    ///
    /// # Arguments
    ///
    /// * `message` - A string slice that holds the message to be logged.
    pub fn log(&self, message: &str) {
        println!("{}: {}", self.junction_id, message);
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
        let package =
            SlowPackage::new_json_payload(recipient_id.clone(), self.junction_id.clone(), &json)
                .expect("Failed to create package");
        queue.push_back(package);
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
    pub async fn get_waiting_package_count(&self) -> usize {
        let queue = self.received_queue.lock().await;
        queue.len()
    }

    /// Returns the number of duplicate packets received & rejected.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of duplicate packets rejected.
    pub fn get_duplicate_package_count(&self) -> usize {
        self.duplicate_package_count.load(Ordering::SeqCst)
    }

    /// Waits for a notification that there are items in the received queue and returns the JSON packet.
    ///
    /// # Returns
    ///
    /// * `Option<JsonPacket>` - An optional JSON packet if available.
    pub async fn wait_for_package(&self) -> Option<JsonPacket> {
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

    /// Updates the known junctions by adding the sender address and sender ID.
    ///
    /// # Arguments
    ///
    /// * `package` - A reference to the `SlowPackage` that was received.
    /// * `sender_addr` - The `SocketAddr` of the sender to be added.
    async fn update_route_table(&self, package: &SlowPackage, sender_addr: SocketAddr) -> u32 {
        let mut known_junctions = self.known_junctions.lock().await;
        known_junctions.insert(sender_addr);

        let junction_id = package.get_sender_id();
        let hop_count = package.get_hop_count();
        let package_id = package.get_package_id();
        let time = 0.0;

        let mut route_table = self.route_table.lock().await;
        route_table.update_route(junction_id, sender_addr, hop_count, time, package_id)
    }

    /// Handles a received package by forwarding it and updating the known junctions and received queue.
    ///
    /// # Arguments
    ///
    /// * `package` - A `SlowPackage` that was received.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn on_package_received(&self, package: SlowPackage, sender_addr: SocketAddr) {
        // Update the route table with the sender address.
        let last_package_id = self.update_route_table(&package, sender_addr).await;

        // If the package has already been processed, return.
        if last_package_id >= package.get_package_id() {
            self.duplicate_package_count.fetch_add(1, Ordering::SeqCst);
            return;
        }

        if *package.get_recipient_id() != self.junction_id {
            self.forward(package, sender_addr).await;
            return;
        }
        if let Some(json) = package.get_json_payload() {
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

    /// Forwards a `SlowPackage` to all known junctions except the sender.
    ///
    /// # Arguments
    ///
    /// * `package` - A `SlowPackage` to be forwarded.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn forward(&self, mut package: SlowPackage, sender_addr: SocketAddr) {
        if package.increment_hops() >= 4 {
            return;
        }
        if self.send_to_best_route(&package).await {
            return;
        }
        self.send_to_known_junctions(package, Some(sender_addr))
            .await;
    }

    /// Sends a `SlowPackage` to all known junctions except the specified sender.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` to be sent.
    /// * `exclude_addr` - The `SocketAddr` of the sender to be excluded.
    pub async fn send_to_known_junctions(
        &self,
        package: SlowPackage,
        exclude_addr: Option<SocketAddr>,
    ) {
        let known_junctions = self.known_junctions.lock().await;
        for addr in known_junctions.iter() {
            if Some(*addr) != exclude_addr {
                self.connection
                    .send_package(&package, addr)
                    .await
                    .expect("Failed to send package");
            }
        }
    }

    /// Waits for a package via `connection.recv_package()` and returns it.
    ///
    /// # Returns
    ///
    /// * `Option<(SlowPackage, SocketAddr)>` - An optional tuple containing the received package and sender address.
    pub async fn read_package(&self) -> Option<(SlowPackage, SocketAddr)> {
        self.connection.recv_package().await
    }

    /// Sends all queued packages to known junctions, excluding the address `0.0.0.0:0`.
    async fn pump_send(&self) {
        self.send_notify.notified().await;
        let mut queue = self.send_queue.lock().await;

        while let Some(mut package) = queue.pop_front() {
            let package_id = self.sent_package_count.fetch_add(1, Ordering::SeqCst) + 1;
            package.set_package_id(package_id);

            if self.send_to_best_route(&package).await {
                continue;
            }

            self.send_to_known_junctions(package, None).await;
        }
    }

    /// Receives a package via `connection.recv_package()` and processes it.
    async fn pump_recv(&self) {
        if let Some((slow_package, sender_addr)) = self.connection.recv_package().await {
            self.on_package_received(slow_package, sender_addr).await;
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

    /// Gets the best route to a junction.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The `JunctionId` of the junction.
    ///
    /// # Returns
    ///
    /// * `Option<SocketAddr>` - The best route to the junction.
    pub async fn get_best_route(&self, junction_id: &JunctionId) -> Option<SocketAddr> {
        let route_table = self.route_table.lock().await;
        let best_route_addr = route_table.get_best_route(junction_id);
        best_route_addr
    }

    /// Sends a `SlowPackage` to the best route available.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` to be sent.
    pub async fn send_to_best_route(&self, package: &SlowPackage) -> bool {
        if let Some(best_route) = self.get_best_route(package.get_recipient_id()).await {
            self.connection
                .send_package(package, &best_route)
                .await
                .expect("Failed to send package");

            return true;
        }

        false
    }
}
