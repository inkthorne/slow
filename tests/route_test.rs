use slow::junction::JunctionId;
use slow::route::RouteTable;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn test_insert_and_get_best_route() {
    let mut route_table = RouteTable::new();
    let junction_id = JunctionId::new("1");
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1111);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2222);

    route_table.insert_route(&junction_id, addr1, 5, 10.0);
    route_table.insert_route(&junction_id, addr2, 3, 5.0);

    let best_route = route_table.get_best_route(&junction_id);
    assert_eq!(best_route, Some(addr2));
}

#[test]
fn test_remove_route() {
    let mut route_table = RouteTable::new();
    let junction_id = JunctionId::new("1");
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1111);

    route_table.insert_route(&junction_id, addr, 5, 10.0);
    let removed_route = route_table.remove(&junction_id);

    assert!(removed_route.is_some());
    assert!(route_table.get_best_route(&junction_id).is_none());
}
