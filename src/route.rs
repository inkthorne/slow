use crate::junction::JunctionId;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct RouteInfo {
    pub hops: u16,
    pub time: f32,
}

pub struct Route {
    routes: HashMap<SocketAddr, RouteInfo>,
}

impl Route {
    pub fn new() -> Self {
        Route {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, addr: SocketAddr, hops: u16, time: f32) {
        self.routes.insert(addr, RouteInfo { hops, time });
    }

    pub fn get_best_route(&self) -> Option<SocketAddr> {
        self.routes
            .iter()
            .min_by_key(|&(_, route_info)| route_info.hops)
            .map(|(&addr, _)| addr)
    }
}

pub struct RouteTable {
    junctions: HashMap<JunctionId, Route>,
}

impl RouteTable {
    pub fn new() -> Self {
        RouteTable {
            junctions: HashMap::new(),
        }
    }

    pub fn insert_route(
        &mut self,
        junction_id: &JunctionId,
        addr: SocketAddr,
        hops: u16,
        time: f32,
    ) {
        let route = self
            .junctions
            .entry(junction_id.clone())
            .or_insert_with(Route::new);
        route.add_route(addr, hops, time);
    }

    pub fn get_best_route(&self, junction_id: &JunctionId) -> Option<SocketAddr> {
        self.junctions
            .get(junction_id)
            .and_then(|route| route.get_best_route())
    }

    pub fn remove(&mut self, junction_id: &JunctionId) -> Option<Route> {
        self.junctions.remove(junction_id)
    }
}
