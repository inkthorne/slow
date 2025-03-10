use crate::junction::JunctionId;
use crate::package::SlowPackage;
use crate::tcp::tcp_link::{SlowLinkId, SlowTcpLink};
use crate::tcp::tcp_router::SlowTcpRouter;
use crate::tracker::UpdateResult;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex, Notify};
use tokio::task;

/// A TCP-based junction that manages multiple TCP links.
///
/// `SlowTcpJunction` is responsible for maintaining connections with multiple
/// TCP endpoints and provides methods to add, remove, and interact with links.
pub struct SlowTcpJunction {
    /// The collection of TCP links managed by this junction
    links: Mutex<Vec<Arc<SlowTcpLink>>>,

    /// A notification mechanism to signal when links are added/removed
    links_changed: Arc<Notify>,

    /// The local socket address this junction is bound to
    local_addr: SocketAddr,

    /// The unique identifier for this junction
    junction_id: JunctionId,

    /// Maps remote junction IDs to their socket addresses
    junction_map: Arc<Mutex<HashMap<JunctionId, SocketAddr>>>,

    /// Counter for the number of packages received
    received_package_count: AtomicUsize,

    /// Counter for the number of packages sent
    sent_package_count: AtomicUsize,

    /// Counter for the number of packages rejected (failed to unpack, duplicate, or old)
    rejected_package_count: AtomicUsize,

    /// Queue of received packages meant for this junction
    received_packages: Mutex<VecDeque<SlowPackage>>,

    /// Routes packages and tracks statistics for different links
    router: Mutex<SlowTcpRouter>,
}

// ---
// SlowTcpJunction: Associated Functions
// ---

impl SlowTcpJunction {
    /// Creates a new `SlowTcpJunction` bound to the specified address.
    ///
    /// # Arguments
    /// * `addr` - The socket address to bind to
    /// * `junction_id` - The unique identifier for this junction
    ///
    /// # Returns
    /// A new SlowTcpJunction instance
    pub fn new(addr: SocketAddr, junction_id: JunctionId) -> Arc<Self> {
        let junction = SlowTcpJunction {
            links: Mutex::new(Vec::new()),
            links_changed: Arc::new(Notify::new()),
            local_addr: addr,
            junction_id,
            junction_map: Arc::new(Mutex::new(HashMap::new())),
            received_package_count: AtomicUsize::new(0),
            sent_package_count: AtomicUsize::new(0),
            rejected_package_count: AtomicUsize::new(0),
            received_packages: Mutex::new(VecDeque::new()),
            router: Mutex::new(SlowTcpRouter::new()),
        };

        let junction = Arc::new(junction);
        junction.clone().start_listening();
        junction
    }
}

// ---
// SlowTcpJunction: Public Functions
// ---

impl SlowTcpJunction {
    /// Connects to a remote junction at the specified address.
    ///
    /// # Arguments
    /// * `addr` - The remote address to connect to
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn connect(self: Arc<Self>, addr: SocketAddr) -> std::io::Result<()> {
        let link = SlowTcpLink::connect(addr).await?;
        let link = Arc::new(link);
        self.add_link(link.clone()).await;
        self.start_processing(link);
        Ok(())
    }

    /// Sends a SlowPackage to connected links.
    ///
    /// This function serializes the provided SlowPackage and sends it to the
    /// appropriate link(s) based on the destination junction ID.
    ///
    /// # Arguments
    /// * `package` - The SlowPackage to send
    ///
    /// # Returns
    /// * `std::io::Result<usize>` - The number of bytes sent or an IO error
    pub async fn send_package(&self, package: &SlowPackage) -> std::io::Result<usize> {
        // Set the package ID to the current sent count before packaging
        let package_id = self.sent_package_count.load(Ordering::Relaxed) as u32 + 1;

        // Serialize the package to bytes
        let data = package.pack(package_id);

        // Use the existing send method to send the data
        let result = self.broadcast(&data, None).await;

        // If send was successful, increment the sent package counter
        if result.is_ok() {
            self.sent_package_count.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Closes all active links in the junction.
    ///
    /// This function attempts to gracefully close all the TCP links managed by this junction.
    /// It returns an error if any of the link closures fail, but attempts to close all links
    /// regardless of individual failures.
    ///
    /// # Returns
    /// * `std::io::Result<()>` - Ok if all links closed successfully, or the last error encountered
    pub async fn close(&self) -> std::io::Result<()> {
        let links_org = self.links.lock().await;
        let mut links = links_org.clone();
        drop(links_org);
        let mut last_error = None;

        // Close all links
        for link in links.iter() {
            if let Err(e) = link.close().await {
                self.log(&format!("Error closing link: {}", e));
                last_error = Some(e);
            }
            self.log("Link closed");
        }

        // Clear the links collection
        links.clear();

        // Notify listeners that links have changed (all removed)
        self.links_changed.notify_one();

        // Return Ok if all links closed successfully, or the last error encountered
        match last_error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Returns the number of active links in this junction.
    pub async fn link_count(&self) -> usize {
        let links = self.links.lock().await;
        links.len()
    }

    /// Returns the local address of this junction.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns the junction ID.
    pub fn junction_id(&self) -> &JunctionId {
        &self.junction_id
    }

    /// Returns the count of packages that have been received.
    pub fn received_package_count(&self) -> usize {
        self.received_package_count.load(Ordering::Relaxed)
    }

    /// Returns the count of packages that have been sent.
    pub fn sent_package_count(&self) -> usize {
        self.sent_package_count.load(Ordering::Relaxed)
    }

    /// Returns the count of packages that have been rejected.
    pub fn rejected_package_count(&self) -> usize {
        self.rejected_package_count.load(Ordering::Relaxed)
    }

    /// Associates a junction ID with a socket address.
    ///
    /// # Arguments
    /// * `junction_id` - The junction ID to map
    /// * `addr` - The socket address to associate with the ID
    pub async fn register_junction(&self, junction_id: JunctionId, addr: SocketAddr) {
        let mut map = self.junction_map.lock().await;
        map.insert(junction_id, addr);
    }

    /// Retrieves the socket address associated with a junction ID.
    ///
    /// # Arguments
    /// * `junction_id` - The junction ID to look up
    ///
    /// # Returns
    /// Option containing the socket address, or None if not found
    pub async fn get_junction_addr(&self, junction_id: &JunctionId) -> Option<SocketAddr> {
        let map = self.junction_map.lock().await;
        map.get(junction_id).copied()
    }

    /// Retrieves the next package from the received packages queue.
    ///
    /// # Returns
    /// Option containing a package, or None if queue is empty
    pub async fn receive_package(&self) -> Option<SlowPackage> {
        let mut packages = self.received_packages.lock().await;
        packages.pop_front()
    }

    /// Returns the number of packages waiting in the received queue.
    ///
    /// # Returns
    /// The count of packages in the received queue
    pub async fn waiting_package_count(&self) -> usize {
        let packages = self.received_packages.lock().await;
        packages.len()
    }
}

// ---
// SlowTcpJunction: Prviate Functions
// ---

impl SlowTcpJunction {
    /// Logs a message prefixed with the junction ID.
    ///
    /// # Arguments
    /// * `message` - The message to log
    fn log(&self, message: &str) {
        println!("[{}]: {}", self.junction_id, message);
    }

    /// Sends data through a specific link.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send
    /// * `link_id` - The ID of the link to send through
    ///
    /// # Returns
    /// * `std::io::Result<usize>` - The number of bytes sent or an IO error
    async fn forward(&self, data: &[u8], link_id: SlowLinkId) -> std::io::Result<usize> {
        let links = self.links.lock().await;
        if let Some(link) = links.iter().find(|l| l.id() == link_id) {
            link.send(data).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Link {} not found", link_id),
            ))
        }
    }

    /// Adds a TCP link to the junction.
    ///
    /// # Arguments
    /// * `link` - The SlowTcpLink to add
    async fn add_link(&self, link: Arc<SlowTcpLink>) {
        let mut links = self.links.lock().await;
        self.log(&format!("link {} added to junction", link.id()));
        links.push(link);
        self.links_changed.notify_one();
    }

    /// Removes a TCP link from the junction.
    ///
    /// # Arguments
    /// * `link_id` - The ID of the SlowTcpLink to remove
    async fn remove_link(&self, link_id: SlowLinkId) {
        let mut links = self.links.lock().await;
        // Find and remove the link with matching ID
        links.retain(|link| link.id() != link_id);
        self.links_changed.notify_one();
        self.log(&format!("link {} removed from junction", link_id));
        self.log(&format!("Link count: {}", links.len()));
    }

    /// Sends data to all connected links except the one specified by exclude_link_id.
    ///
    /// This function broadcasts the provided data to all active links managed by this junction,
    /// excluding the link specified by exclude_link_id if provided.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send
    /// * `exclude_link_id` - Optional link ID to exclude from broadcasting
    ///
    /// # Returns
    /// * `std::io::Result<usize>` - The number of bytes sent or an IO error
    async fn broadcast(
        &self,
        data: &[u8],
        exclude_link_id: Option<SlowLinkId>,
    ) -> std::io::Result<usize> {
        self.log(&format!(
            "Broadcasting data to all links (excluding {})",
            exclude_link_id.unwrap_or(0)
        ));

        // Get a reference to all active links
        let links = self.links.lock().await;

        if links.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "No active links available for sending data",
            ));
        }

        let mut last_error = None;
        let mut bytes_sent = 0;

        // Send the data to all links except the excluded one
        for link in links.iter() {
            // Skip if this is the excluded link
            if let Some(excluded_id) = exclude_link_id {
                if link.id() == excluded_id {
                    continue;
                }
            }

            match link.send(data).await {
                Ok(sent) => {
                    // Return the number of bytes sent on the first successful transmission
                    if bytes_sent == 0 {
                        bytes_sent = sent;
                    }
                }
                Err(e) => {
                    // Store the error but continue trying other links
                    self.log(&format!("Error sending data on link: {}", e));
                    last_error = Some(e);
                }
            }
        }

        // If we sent data on at least one link, consider it a success
        if bytes_sent > 0 {
            Ok(bytes_sent)
        } else {
            // If all links failed, return the last error
            Err(last_error.unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to send data on any link")
            }))
        }
    }

    /// Starts listening for incoming connections on the local address.
    ///
    /// This method spawns a background task that continuously listens for
    /// incoming connections on the local address specified during junction creation.
    /// Each accepted connection is added as a new link to the junction.
    fn start_listening(self: Arc<Self>) {
        // Spawn a tokio backgrond task to handle incoming connections
        task::spawn(async move {
            loop {
                match SlowTcpLink::listen(self.local_addr).await {
                    Ok(link) => {
                        // Add the new link to the junction
                        let link = Arc::new(link);
                        self.add_link(link.clone()).await;
                        self.clone().start_processing(link);
                    }
                    Err(e) => {
                        self.log(&format!("Error accepting connection: {}", e));
                        // Small delay to avoid tight loop on persistent errors
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
    }

    /// Starts processing data from a newly established TCP link.
    ///
    /// This method spawns a background task to continuously receive and process data
    /// from the provided TCP link. It handles incoming data and any errors that occur
    /// during communication.
    ///
    /// # Arguments
    /// * `link` - The SlowTcpLink to process data from
    fn start_processing(self: Arc<Self>, link: Arc<SlowTcpLink>) {
        // Spawn a tokio background task to handle incoming connections
        task::spawn(async move {
            let mut buffer = vec![0u8; SlowTcpLink::max_frame_size()];
            loop {
                match link.receive(&mut buffer).await {
                    Ok(size) => {
                        if size == 0 {
                            self.remove_link(link.id()).await;
                            break;
                        }
                        let data = &buffer[..size];
                        self.process(data, link.id()).await;
                    }
                    Err(_) => {
                        self.log("Error receiving data from link");
                        break;
                    }
                }
            }

            self.remove_link(link.id()).await;
            self.log("Link processing task finished");
        });
    }

    /// Processes received data from a TCP link.
    ///
    /// This function unpacks the received data into a SlowPackage and checks if it's intended
    /// for this junction. If it is, the package is stored in a Deque for later processing.
    ///
    /// # Arguments
    /// * `data` - The slice of bytes received from the link
    /// * `link_id` - The ID of the link that received the data
    async fn process(&self, data: &[u8], link_id: SlowLinkId) {
        // Try to unpack the data into a SlowPackage
        let package = match SlowPackage::unpack(data) {
            Some(package) => package,
            None => {
                self.log(&format!(
                    "Failed to unpack received data from link {}: {:?}",
                    link_id, data
                ));
                self.rejected_package_count.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let recipient_id = package.recipient_id();

        // Check the package against the router with the link_id
        let best_link = {
            let mut router = self.router.lock().await;
            match router.update(&package, link_id) {
                UpdateResult::Duplicate | UpdateResult::Old => {
                    self.log(&format!(
                        "Received old or duplicate package {} from {} for {}",
                        package.package_id(),
                        package.sender_id(),
                        recipient_id
                    ));
                    self.rejected_package_count.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                UpdateResult::Success => {
                    // Package is new and valid, continue processing
                    self.log(&format!(
                        "Received new package {} from {} for {}",
                        package.package_id(),
                        package.sender_id(),
                        recipient_id
                    ));
                }
            }

            router.get_best_link(recipient_id)
        };

        // Increment the received package counter
        self.received_package_count.fetch_add(1, Ordering::Relaxed);

        // Check if the package is intended for this junction
        if *recipient_id == self.junction_id {
            // Lock the deque and add the package
            let mut received_packages = self.received_packages.lock().await;
            self.log("Package is for this junction, saving to queue");
            received_packages.push_back(package);
        } else {
            if best_link.is_some() {
                let best_link = best_link.unwrap();
                self.log(&format!(
                    "Forwarding package through best link {}",
                    best_link
                ));

                let result = self.forward(&data, best_link).await;
                if result.is_ok() {
                    self.sent_package_count.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            }

            self.log("No best link found or best link failed; broadcasting to all links");
            self.broadcast(&data, Some(link_id))
                .await
                .unwrap_or_else(|e| {
                    self.log(&format!("Failed to broadcast: {}", e));
                    0
                });
        }
    }
}
