use crate::junction::JunctionId;
use crate::tcp::tcp_link::SlowTcpLink;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::task;

/// A TCP-based junction that manages multiple TCP links.
///
/// `SlowTcpJunction` is responsible for maintaining connections with multiple
/// TCP endpoints and provides methods to add, remove, and interact with links.
pub struct SlowTcpJunction {
    /// The collection of TCP links managed by this junction
    links: Arc<Mutex<Vec<SlowTcpLink>>>,

    /// A notification mechanism to signal when links are added/removed
    links_changed: Arc<Notify>,

    /// The local socket address this junction is bound to
    local_addr: SocketAddr,

    /// The unique identifier for this junction
    junction_id: JunctionId,

    /// Maps remote junction IDs to their socket addresses
    junction_map: Arc<Mutex<HashMap<JunctionId, SocketAddr>>>,
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
            links: Arc::new(Mutex::new(Vec::new())),
            links_changed: Arc::new(Notify::new()),
            local_addr: addr,
            junction_id,
            junction_map: Arc::new(Mutex::new(HashMap::new())),
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
    pub async fn connect(&self, addr: SocketAddr) -> std::io::Result<()> {
        let link = SlowTcpLink::connect(addr).await?;
        self.add_link(link);
        Ok(())
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
}

// ---
// SlowTcpJunction: Prviate Functions
// ---

impl SlowTcpJunction {
    /// Adds a TCP link to the junction.
    ///
    /// # Arguments
    /// * `link` - The SlowTcpLink to add
    fn add_link(&self, link: SlowTcpLink) {
        let mut links = self.links.lock().unwrap();
        links.push(link);
        self.links_changed.notify_one();
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
                        self.clone().start_processing(link);
                    }
                    Err(e) => {
                        eprintln!("SlowTcpJunction: error accepting connection: {}", e);
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
    fn start_processing(self: Arc<Self>, link: SlowTcpLink) {
        // Spawn a tokio background task to handle incoming connections
        task::spawn(async move {
            let mut buffer = Vec::with_capacity(SlowTcpLink::max_frame_size());
            loop {
                match link.receive(&mut buffer).await {
                    Ok(size) => {
                        let _data = &buffer[..size];
                    }
                    Err(e) => {
                        eprintln!("SlowTcpJunction: error on receive: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
