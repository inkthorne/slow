use serde_json::json;
use std::net::SocketAddr;
mod connection;
pub mod datagram;
pub mod junction;

fn main() {
    println!("Hello, world!");
    let junction1_addr: SocketAddr = "[::1]:1111".parse().expect("Invalid address");
    let junction1 =
        junction::SlowJunction::new(junction1_addr, 1).expect("Couldn't create SlowJunction");

    // Create a second junction and seed it with the first junction's address
    let junction2_addr: SocketAddr = "[::1]:2222".parse().expect("Invalid address");
    let junction2 =
        junction::SlowJunction::new(junction2_addr, 2).expect("Couldn't create SlowJunction");
    junction1.seed(junction2_addr);

    // Create a third junction with port 3333
    let junction3_addr: SocketAddr = "[::1]:3333".parse().expect("Invalid address");
    let junction3 =
        junction::SlowJunction::new(junction3_addr, 3).expect("Couldn't create SlowJunction");
    junction2.seed(junction3_addr);
    junction3.seed(junction1_addr);

    let json_message = json!({
        "message": "Hello from sender!",
        "slow": "0.1"
    });
    junction1.send(json_message, 3);

    // Keep the main thread alive to allow other threads to run
    loop {
        if let Some(packet) = junction1.recv() {
            println!(
                "Received packet on port {}: {:?}",
                junction1.get_address().port(),
                packet.json
            );
        }
        if let Some(packet) = junction2.recv() {
            println!(
                "Received packet on port {}: {:?}",
                junction2.get_address().port(),
                packet.json
            );
        }
        if let Some(packet) = junction3.recv() {
            println!(
                "Received packet on port {}: {:?}",
                junction3.get_address().port(),
                packet.json
            );
        }
    }
}
