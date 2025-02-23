use crate::junction::JunctionId;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Represents information about a route package, including the greatest package ID and a bitfield for package tracking.
///
/// The `RoutePackageInfo` struct is used to manage and update package information for routes.
/// It keeps track of the greatest package ID and uses a bitfield to track package updates.
///
/// # Fields
///
/// * `greatest_package_id` - The ID of the greatest package.
/// * `package_bitfield` - A bitfield used to track package updates.
pub struct RoutePackageInfo {
    /// The ID of the greatest package.
    greatest_package_id: u32,

    /// A bitfield used to track package updates.
    package_bitfield: u32,
}

impl RoutePackageInfo {
    /// Creates a new `RoutePackageInfo`.
    ///
    /// # Returns
    ///
    /// A new instance of `RoutePackageInfo` with default values.
    pub fn new() -> Self {
        RoutePackageInfo {
            greatest_package_id: 0,
            package_bitfield: 0,
        }
    }

    /// Updates the package information with a new package ID.
    ///
    /// # Arguments
    ///
    /// * `package_id` - The ID of the package to update.
    ///
    /// # Returns
    ///
    /// `true` if the package information was updated successfully, `false` otherwise.
    pub fn update(&mut self, package_id: u32) -> bool {
        let shift = package_id as i32 - self.greatest_package_id as i32;

        if shift == 0 || shift < -31 {
            return false;
        }

        if shift < 0 {
            let mask = 1 << -shift;

            if self.package_bitfield & mask != 0 {
                return false;
            }

            self.package_bitfield |= mask;
            return true;
        }

        if shift > 31 {
            self.package_bitfield = 0;
            self.package_bitfield |= 1;
        } else {
            self.package_bitfield <<= shift;
            self.package_bitfield |= 1;
        }

        self.greatest_package_id = package_id;

        true
    }
}

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
    greatest_package_id: u32,

    package_info: RoutePackageInfo,
}

impl Route {
    /// Creates a new `Route`.
    pub fn new() -> Self {
        Route {
            routes: HashMap::new(),
            greatest_package_id: 0,
            package_info: RoutePackageInfo::new(),
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
        self.package_info.update(package_id);

        if package_id > self.greatest_package_id {
            let previous_greatest_package_id = self.greatest_package_id;
            self.greatest_package_id = package_id;

            return previous_greatest_package_id;
        }

        self.greatest_package_id
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
