use crate::junction::JunctionId;
use crate::package::SlowPackage;
use crate::package_tracker::SlowPackageTracker;
use crate::tcp::tcp_link::SlowTcpLink;
use crate::tracker::UpdateResult;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
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

    /// Queue of received packages meant for this junction
    received_packages: Mutex<VecDeque<SlowPackage>>,

    /// Tracks packages to detect duplicates and old packages
    package_tracker: Mutex<SlowPackageTracker>,
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
            received_packages: Mutex::new(VecDeque::new()),
            package_tracker: Mutex::new(SlowPackageTracker::new()),
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
        self.add_link(link.clone());
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
        let result = self.send(&data).await;

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
        let links_org = self.links.lock().unwrap();
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
    pub fn link_count(&self) -> usize {
        let links = self.links.lock().unwrap();
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

    /// Associates a junction ID with a socket address.
    ///
    /// # Arguments
    /// * `junction_id` - The junction ID to map
    /// * `addr` - The socket address to associate with the ID
    pub fn register_junction(&self, junction_id: JunctionId, addr: SocketAddr) {
        let mut map = self.junction_map.lock().unwrap();
        map.insert(junction_id, addr);
    }

    /// Retrieves the socket address associated with a junction ID.
    ///
    /// # Arguments
    /// * `junction_id` - The junction ID to look up
    ///
    /// # Returns
    /// Option containing the socket address, or None if not found
    pub fn get_junction_addr(&self, junction_id: &JunctionId) -> Option<SocketAddr> {
        let map = self.junction_map.lock().unwrap();
        map.get(junction_id).copied()
    }

    /// Retrieves the next package from the received packages queue.
    ///
    /// # Returns
    /// Option containing a package, or None if queue is empty
    pub fn receive_package(&self) -> Option<SlowPackage> {
        let mut packages = self.received_packages.lock().unwrap();
        packages.pop_front()
    }

    /// Returns the number of packages waiting in the received queue.
    ///
    /// # Returns
    /// The count of packages in the received queue
    pub fn waiting_package_count(&self) -> usize {
        let packages = self.received_packages.lock().unwrap();
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

    /// Adds a TCP link to the junction.
    ///
    /// # Arguments
    /// * `link` - The SlowTcpLink to add
    fn add_link(&self, link: Arc<SlowTcpLink>) {
        let mut links = self.links.lock().unwrap();
        self.log(&format!("link {} added to junction", link.id()));
        links.push(link);
        self.links_changed.notify_one();
    }

    /// Removes a TCP link from the junction.
    ///
    /// # Arguments
    /// * `link_id` - The ID of the SlowTcpLink to remove
    fn remove_link(&self, link_id: u64) {
        let mut links = self.links.lock().unwrap();
        // Find and remove the link with matching ID
        links.retain(|link| link.id() != link_id);
        self.links_changed.notify_one();
        self.log(&format!("link {} removed from junction", link_id));
        self.log(&format!("Link count: {}", links.len()));
    }

    /// Sends data to all connected links.
    ///
    /// This function broadcasts the provided data to all active links managed by this junction.
    ///
    /// # Arguments
    /// * `data` - The byte slice to send
    ///
    /// # Returns
    /// * `std::io::Result<usize>` - The number of bytes sent or an IO error
    async fn send(&self, data: &[u8]) -> std::io::Result<usize> {
        // Get a reference to all active links
        let links = self.links.lock().unwrap();

        if links.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "No active links available for sending data",
            ));
        }

        let mut last_error = None;
        let mut bytes_sent = 0;

        // Send the data to all links
        for link in links.iter() {
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
                        self.add_link(link.clone());
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
                self.log(&format!("Waiting to receive data from link {}", link.id()));
                match link.receive(&mut buffer).await {
                    Ok(size) => {
                        self.log(&format!("Received {} bytes from link", size));
                        if size == 0 {
                            self.remove_link(link.id());
                            break;
                        }
                        let data = &buffer[..size];
                        self.process(data).await;
                    }
                    Err(_) => {
                        self.log("Error receiving data from link");
                        break;
                    }
                }
            }

            self.remove_link(link.id());
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
    async fn process(&self, data: &[u8]) {
        // Try to unpack the data into a SlowPackage
        let package = match SlowPackage::unpack(data) {
            Some(package) => package,
            None => {
                self.log(&format!("Failed to unpack received data: {:?}", data));
                return;
            }
        };

        // Check the package against the tracker
        let mut tracker = self.package_tracker.lock().unwrap();
        match tracker.update(&package) {
            UpdateResult::Duplicate => {
                self.log("Received duplicate package, discarding");
                return;
            }
            UpdateResult::Old => {
                self.log("Received old package, discarding");
                return;
            }
            UpdateResult::Success => {
                // Package is new and valid, continue processing
                self.log(&format!(
                    "Received new package {} from {} for {}",
                    package.package_id(),
                    package.sender_id(),
                    package.recipient_id()
                ));
            }
        }
        drop(tracker);

        // Increment the received package counter
        self.received_package_count.fetch_add(1, Ordering::Relaxed);

        // Check if the package is intended for this junction
        if *package.recipient_id() == self.junction_id {
            // Lock the deque and add the package
            let mut received_packages = self.received_packages.lock().unwrap();
            self.log("Package is for this junction, saving to queue");
            received_packages.push_back(package);
        } else {
            self.log("Package is not for this junction, forwarding.");
            // self.send(&data).await;
        }
    }
}
