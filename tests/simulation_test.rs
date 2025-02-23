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
        let mut slow_junctions = Vec::with_capacity(num_junctions);
        let mut rng = rand::thread_rng();
        for _ in 0..num_junctions {
            let port: u16 = rng.gen_range(1024..65535);
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
            let id = format!("junction-{}", rng.gen::<u32>());
            let junction_id = JunctionId::new(&id);
            let slow_junction = SlowJunction::new(addr, junction_id).await.unwrap();
            slow_junctions.push(slow_junction);
        }
        Self {
            junctions: slow_junctions,
        }
    }

    pub async fn seed_junctions(&self) {
        let mut rng = rand::thread_rng();
        for junction in &self.junctions {
            let random_junction = &self.junctions[rng.gen_range(0..self.junctions.len())];
            junction.seed(random_junction.get_address()).await;
        }
    }

    pub async fn ping(&self) {
        let mut rng = rand::thread_rng();
        let source_junction = &self.junctions[rng.gen_range(0..self.junctions.len())];
        let target_junction = &self.junctions[rng.gen_range(0..self.junctions.len())];

        source_junction.seed(target_junction.get_address()).await;
        source_junction
            .ping(&target_junction.get_junction_id())
            .await;
        sleep(Duration::from_millis(1250)).await;
        let pong_count = target_junction.get_pong_counter().await;
        assert!(pong_count > 0, "Target junction did not receive the ping");
    }
}

#[tokio::test]
async fn test_simulation() {
    let simulation = JunctionSimulation::new(5).await;
    simulation.seed_junctions().await;
    simulation.ping().await;
}
