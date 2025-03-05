use crate::junction::JunctionId;
use crate::package::SlowPackage;
use crate::tcp::tcp_link::SlowTcpLink;
use std::collections::HashMap;
use std::io::{self, Error};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

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

impl SlowTcpJunction {
    /// Creates a new `SlowTcpJunction` bound to the specified address.
    ///
    /// # Arguments
    /// * `addr` - The socket address to bind to
    /// * `junction_id` - The unique identifier for this junction
    ///
    /// # Returns
    /// A new SlowTcpJunction instance
    pub fn new(addr: SocketAddr, junction_id: JunctionId) -> Self {
        SlowTcpJunction {
            links: Arc::new(Mutex::new(Vec::new())),
            links_changed: Arc::new(Notify::new()),
            local_addr: addr,
            junction_id,
            junction_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Adds a TCP link to the junction.
    ///
    /// # Arguments
    /// * `link` - The SlowTcpLink to add
    pub fn add_link(&self, link: SlowTcpLink) {
        let mut links = self.links.lock().unwrap();
        links.push(link);
        self.links_changed.notify_one();
    }

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

    /// Waits for a package to be received on any of the links managed by this junction.
    ///
    /// This function polls all links in a round-robin fashion until a valid package is received.
    /// If a link is disconnected or encounters an error, it continues with other links.
    ///
    /// # Returns
    /// * `io::Result<SlowPackage>` - The received package, or an error if all links fail
    ///
    /// # Errors
    /// Returns an error if no links are available or all links fail to receive data
    pub async fn wait_for_package(&self) -> io::Result<SlowPackage> {
        // use futures::future::{FutureExt, select_ok};
        Err(Error::new(io::ErrorKind::Other, "Not implemented"))
    }
}
