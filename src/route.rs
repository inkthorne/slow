use crate::junction::JunctionId;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Represents information about a route, including the number of hops and the time taken.
pub struct RouteInfo {
    /// The number of hops to reach the destination.
    pub hops: u8,

    /// The time taken to reach the destination.
    pub time: f32,
}

/// Represents a collection of routes and manages route updates and retrievals.
pub struct Route {
    /// A map of socket addresses to route information.
    routes: HashMap<SocketAddr, RouteInfo>,

    /// The ID of the last package.
    last_package_id: u32,
}

impl Route {
    /// Creates a new `Route`.
    pub fn new() -> Self {
        Route {
            routes: HashMap::new(),
            last_package_id: 0,
        }
    }

    /// Updates the route information for a given address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address of the route.
    /// * `hops` - The number of hops to reach the destination.
    /// * `time` - The time taken to reach the destination.
    /// * `package_id` - The ID of the package.
    ///
    /// # Returns
    ///
    /// The previous package ID if the new package ID is greater than the last package ID, otherwise the last package ID.
    pub fn update_route(&mut self, addr: SocketAddr, hops: u8, time: f32, package_id: u32) -> u32 {
        self.routes.insert(addr, RouteInfo { hops, time });

        if package_id > self.last_package_id {
            let previous_package_id = self.last_package_id;
            self.last_package_id = package_id;

            return previous_package_id;
        }

        self.last_package_id
    }

    /// Gets the best route with the minimum number of hops.
    ///
    /// # Returns
    ///
    /// An `Option` containing the socket address of the best route, or `None` if no routes are available.
    pub fn get_best_route(&self) -> Option<SocketAddr> {
        self.routes
            .iter()
            .min_by_key(|&(_, route_info)| route_info.hops)
            .map(|(&addr, _)| addr)
    }
}

/// Represents a table of routes for different junctions and manages route updates and retrievals.
pub struct RouteTable {
    /// A map of junction IDs to routes.
    junctions: HashMap<JunctionId, Route>,
}

impl RouteTable {
    /// Creates a new `RouteTable`.
    pub fn new() -> Self {
        RouteTable {
            junctions: HashMap::new(),
        }
    }

    /// Updates the route information for a given junction.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The ID of the junction.
    /// * `addr` - The socket address of the route.
    /// * `hops` - The number of hops to reach the destination.
    /// * `time` - The time taken to reach the destination.
    /// * `package_id` - The ID of the package.
    ///
    /// # Returns
    ///
    /// The previous package ID if the new package ID is greater than the last package ID, otherwise the last package ID.
    pub fn update_route(
        &mut self,
        junction_id: &JunctionId,
        addr: SocketAddr,
        hops: u8,
        time: f32,
        package_id: u32,
    ) -> u32 {
        let route = self
            .junctions
            .entry(junction_id.clone())
            .or_insert_with(Route::new);

        route.update_route(addr, hops, time, package_id)
    }

    /// Gets the best route for a given junction with the minimum number of hops.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The ID of the junction.
    ///
    /// # Returns
    ///
    /// An `Option` containing the socket address of the best route, or `None` if no routes are available.
    pub fn get_best_route(&self, junction_id: &JunctionId) -> Option<SocketAddr> {
        self.junctions
            .get(junction_id)
            .and_then(|route| route.get_best_route())
    }

    /// Removes the route information for a given junction.
    ///
    /// # Arguments
    ///
    /// * `junction_id` - The ID of the junction.
    ///
    /// # Returns
    ///
    /// An `Option` containing the removed `Route`, or `None` if the junction was not found.
    pub fn remove(&mut self, junction_id: &JunctionId) -> Option<Route> {
        self.junctions.remove(junction_id)
    }
}
