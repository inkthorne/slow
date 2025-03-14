// Re-export JunctionId so it can be imported from this module
pub use crate::junction_id::JunctionId;
use crate::package::{PackageType, SlowPackage};
use crate::route::RouteTable;
use crate::udp::udp_socket::SlowUdpSocket;
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use tokio::sync::{Mutex, Notify};

#[derive(Clone, PartialEq, Debug)]
pub struct JsonPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

/// A `SlowJunction` represents a network junction that can send and receive packages, manage known junctions,
/// and handle JSON packets.
///
/// This struct provides methods to create a new junction, send and receive JSON packets, manage known junctions,
/// and run the main loop. It is designed to work asynchronously using the Tokio runtime.
pub struct SlowJunction {
    /// The connection used by the junction.
    connection: SlowUdpSocket,

    /// A set of known junction addresses.
    known_junctions: Mutex<HashSet<SocketAddr>>,

    /// A queue of packages to be sent.
    send_queue: Mutex<VecDeque<SlowPackage>>,

    /// A queue of received JSON packets.
    received_queue: Mutex<VecDeque<JsonPacket>>,

    /// The address of the junction.
    addr: SocketAddr,

    /// The recipient ID for the junction.
    junction_id: JunctionId,

    /// A flag to indicate if the thread should terminate.
    terminate: AtomicBool,

    /// A notification to signal when a package is added to the send queue.
    send_notify: Notify,

    /// A notification to signal when a package is added to the receive queue.
    receive_notify: Notify,

    /// A counter for the number of pong messages received.
    pong_counter: AtomicU32,

    /// The route table for the junction.
    route_table: Mutex<RouteTable>,

    /// A counter for the number of packages sent.
    sent_package_count: AtomicU32,

    /// A counter for the number of unique packages received.
    unique_package_count: AtomicU32,

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
        let connection = SlowUdpSocket::new(addr).await?;
        let junction = Arc::new(Self {
            connection,
            known_junctions: Mutex::new(HashSet::new()),
            send_queue: Mutex::new(VecDeque::new()),
            received_queue: Mutex::new(VecDeque::new()),
            addr,
            junction_id, // use passed JunctionId directly
            terminate: AtomicBool::new(false),
            send_notify: Notify::new(),
            receive_notify: Notify::new(),
            pong_counter: AtomicU32::new(0),
            route_table: Mutex::new(RouteTable::new()),
            sent_package_count: AtomicU32::new(0),
            duplicate_package_count: AtomicUsize::new(0),
            unique_package_count: AtomicU32::new(0),
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
            SlowPackage::new_json_payload(recipient_id.clone(), self.junction_id.clone(), &json);
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
        if addr == self.addr {
            return;
        }
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

    /// Returns the number of unique packages received.
    ///
    /// # Returns
    ///
    /// * `u32` - The number of unique packages received.
    pub fn get_unique_package_count(&self) -> u32 {
        self.unique_package_count.load(Ordering::SeqCst)
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
    async fn update_route_table(&self, package: &SlowPackage, sender_addr: SocketAddr) -> bool {
        let mut known_junctions = self.known_junctions.lock().await;
        known_junctions.insert(sender_addr);

        let junction_id = package.sender_id();
        let hop_count = package.hop_count();
        let package_id = package.package_id();
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
        let package_type = package.package_type();

        if package_type == Ok(PackageType::Hello) {
            self.on_hello_received(package, sender_addr).await;
            return;
        }

        // Update the route table with the sender address.
        let is_updated = self.update_route_table(&package, sender_addr).await;

        // If the package has already been processed, return.
        if !is_updated {
            self.duplicate_package_count.fetch_add(1, Ordering::SeqCst);
            return;
        }

        // Increment unique_package_count for each non-rejected package received.
        self.unique_package_count.fetch_add(1, Ordering::SeqCst);

        if *package.recipient_id() != self.junction_id {
            self.forward(package, sender_addr).await;
            return;
        }

        match package_type {
            Ok(PackageType::Ping) => {
                self.on_ping_received(package).await;
            }
            Ok(PackageType::Pong) => {
                self.on_pong_received().await;
            }
            Ok(PackageType::Json) => {
                if let Some(json) = package.json_payload() {
                    let json_packet = JsonPacket {
                        addr: sender_addr,
                        json,
                    };

                    let mut queue = self.received_queue.lock().await;
                    queue.push_back(json_packet);
                    self.receive_notify.notify_one();
                }
            }
            Ok(PackageType::Bin) => {}
            _ => {}
        }
    }

    /// Forwards a `SlowPackage` to all known junctions except the sender.
    ///
    /// # Arguments
    ///
    /// * `package` - A `SlowPackage` to be forwarded.
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn forward(&self, mut package: SlowPackage, sender_addr: SocketAddr) {
        if package.increment_hops() >= 128 {
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
        self.connection.receive_package().await
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
        if let Some((slow_package, sender_addr)) = self.connection.receive_package().await {
            self.on_package_received(slow_package, sender_addr).await;
        }
    }

    /// Sends a pong message to a specific `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - A u16 representing the recipient ID.
    pub async fn pong(&self, recipient_id: &JunctionId) {
        let mut queue = self.send_queue.lock().await;
        let package = SlowPackage::new_pong(recipient_id.clone(), self.junction_id.clone());
        queue.push_back(package);
        self.send_notify.notify_one();
    }

    /// Sends a ping message to a specific `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The target junction id as a &str.
    pub async fn ping(&self, junction_id: &JunctionId) {
        let mut queue = self.send_queue.lock().await;
        let package = SlowPackage::new_ping(junction_id.clone(), self.junction_id.clone());
        queue.push_back(package);
        self.send_notify.notify_one();
    }

    /// Returns the current value of the pong counter.
    ///
    /// # Returns
    ///
    /// * `u32` - The current value of the pong counter.
    pub async fn get_pong_counter(&self) -> u32 {
        self.pong_counter.load(Ordering::SeqCst)
    }

    /// Increments the pong counter when a pong message is received.
    async fn on_pong_received(&self) {
        self.pong_counter.fetch_add(1, Ordering::SeqCst);
    }

    /// Handles a received ping message by sending a pong response.
    ///
    /// # Arguments
    ///
    /// * `package` - The `SlowPackage` that was received.
    async fn on_ping_received(&self, package: SlowPackage) {
        let sender_id = package.sender_id();
        self.pong(&sender_id).await;
    }

    /// Handles a received hello message by sending a hello response.
    ///
    /// # Arguments
    ///
    /// * `sender_addr` - The `SocketAddr` of the sender.
    async fn on_hello_received(&self, package: SlowPackage, sender_addr: SocketAddr) {
        if package.package_id() == 0 {
            self.send_hello_response(sender_addr).await;
        }
        self.known_junctions.lock().await.insert(sender_addr);
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
        if let Some(best_route) = self.get_best_route(package.recipient_id()).await {
            self.connection
                .send_package(package, &best_route)
                .await
                .expect("Failed to send package");

            return true;
        }

        false
    }

    /// Returns the junction ID.
    pub fn get_junction_id(&self) -> &JunctionId {
        &self.junction_id
    }

    /// Joins a junction by sending a hello message to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The `SocketAddr` to send the hello message to.
    pub async fn join(&self, addr: SocketAddr) {
        self.send_hello(addr).await;
    }

    /// Sends a hello message to a specific `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `addr` - The `SocketAddr` to send the hello message to.
    async fn send_hello(&self, addr: SocketAddr) {
        let package = SlowPackage::new_hello(0, self.junction_id.clone());
        self.connection
            .send_package(&package, &addr)
            .await
            .expect("Failed to send hello package.");
    }

    /// Sends a hello message to a specific `SocketAddr`.
    ///
    /// # Arguments
    ///
    /// * `addr` - The `SocketAddr` to send the hello message to.
    async fn send_hello_response(&self, addr: SocketAddr) {
        let package = SlowPackage::new_hello(1, self.junction_id.clone());
        self.connection
            .send_package(&package, &addr)
            .await
            .expect("Failed to send hello package.");
    }
}
