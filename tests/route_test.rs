use slow::junction::JunctionId;
use slow::route::{RoutePackageInfo, RouteTable};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn test_route_info() {
    let mut route_package_info = RoutePackageInfo::new();
    let success = route_package_info.update(132);
    assert!(success == true);

    let success = route_package_info.update(3);
    assert!(success == false);

    let success = route_package_info.update(101);
    assert!(success == true);

    let success = route_package_info.update(100);
    assert!(success == false);

    let success = route_package_info.update(33);
    assert!(success == false);

    let success = route_package_info.update(134);
    assert!(success == true);

    let success = route_package_info.update(133);
    assert!(success == true);

    let success = route_package_info.update(132);
    assert!(success == false);
}

#[test]
fn test_update_and_get_best_route() {
    let mut route_table = RouteTable::new();
    let junction_id = JunctionId::new("1");
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1111);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2222);

    route_table.update_route(&junction_id, addr1, 5, 10.0, 0);
    route_table.update_route(&junction_id, addr2, 3, 5.0, 0);

    let best_route = route_table.get_best_route(&junction_id);
    assert_eq!(best_route, Some(addr2));

    // Test that no best route is found for a junction_id that doesn't exist in the RouteTable
    let non_existent_junction_id = JunctionId::new("2");
    let best_route_non_existent = route_table.get_best_route(&non_existent_junction_id);
    assert_eq!(best_route_non_existent, None);
}

#[test]
fn test_remove_route() {
    let mut route_table = RouteTable::new();
    let junction_id = JunctionId::new("1");
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1111);

    route_table.update_route(&junction_id, addr, 5, 10.0, 0);
    let removed_route = route_table.remove(&junction_id);

    assert!(removed_route.is_some());
    assert!(route_table.get_best_route(&junction_id).is_none());
}
