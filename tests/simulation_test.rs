use rand::Rng;
use slow::junction::{JunctionId, SlowJunction};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

struct JunctionSimulation {
    junctions: Vec<Arc<SlowJunction>>,
}

impl JunctionSimulation {
    pub async fn new(num_junctions: usize) -> Self {
        let mut slow_junctions: Vec<Arc<SlowJunction>> = Vec::with_capacity(num_junctions);
        let mut rng = rand::thread_rng();
        for i in 0..num_junctions {
            let port: u16 = 1024 + i as u16;
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
            let id = format!("junction-{}", port);
            let junction_id = JunctionId::new(&id);
            let slow_junction = SlowJunction::new(addr, junction_id).await.unwrap();
            if !slow_junctions.is_empty() {
                let random_junction = &slow_junctions[rng.gen_range(0..slow_junctions.len())];
                random_junction.seed(slow_junction.get_address()).await;
                slow_junction.seed(random_junction.get_address()).await;
            }
            slow_junctions.push(slow_junction);
        }
        Self {
            junctions: slow_junctions,
        }
    }

    pub async fn ping(&self) {
        let mut rng = rand::thread_rng();
        let source_junction = &self.junctions[rng.gen_range(0..self.junctions.len())];
        let target_junction = &self.junctions[rng.gen_range(0..self.junctions.len())];

        source_junction
            .ping(&target_junction.get_junction_id())
            .await;

        sleep(Duration::from_millis(250)).await;

        let pong_count = source_junction.get_pong_counter().await;
        assert_eq!(pong_count, 1);
    }
}

#[tokio::test]
async fn test_simulation() {
    let junction_count = 128;
    let simulation = JunctionSimulation::new(junction_count).await;
    simulation.ping().await;

    let mut count_0_package = 0;
    let mut count_1_package = 0;
    let mut count_2_packages = 0;
    let mut count_3_or_more_packages = 0;

    for junction in &simulation.junctions {
        let unique_package_count = junction.get_unique_package_count();
        match unique_package_count {
            0 => count_0_package += 1,
            1 => count_1_package += 1,
            2 => count_2_packages += 1,
            _ => count_3_or_more_packages += 1,
        }
    }

    println!(
        "Number of junctions with 0 unique packages: {}",
        count_0_package
    );

    println!(
        "Number of junctions with 1 unique package: {}",
        count_1_package
    );

    println!(
        "Number of junctions with 2 unique packages: {}",
        count_2_packages
    );

    println!(
        "Number of junctions with 3 or more unique packages: {}",
        count_3_or_more_packages
    );

    assert!(
        count_1_package > count_2_packages,
        "Number of junctions with 1 unique package ({}) is not greater than the number of junctions with 2 unique packages ({})",
        count_1_package,
        count_2_packages
    );

    assert_eq!(count_3_or_more_packages, 0);
    assert_eq!(
        count_0_package + count_1_package + count_2_packages + count_3_or_more_packages,
        junction_count
    );
}
