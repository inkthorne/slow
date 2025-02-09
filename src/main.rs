use serde_json::json;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
mod connection;
pub mod junction;

fn main() {
    println!("Hello, world!");
    let addr: SocketAddr = "[::1]:2222".parse().expect("Invalid address");
    let junction1 = junction::SlowJunction::new(addr).expect("Couldn't create SlowJunction");

    // Create a second junction and seed it with the first junction's address
    let junction2_addr: SocketAddr = "[::1]:3333".parse().expect("Invalid address");
    let junction2 = junction::SlowJunction::new(junction2_addr).expect("Couldn't create SlowJunction");
    junction2.seed(addr);

    thread::spawn(move || {
        loop {
            let json_message = json!({
                "message": "Hello from sender!",
                "slow": "0.1"
            });
            junction2.send(json_message);
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Keep the main thread alive to allow other threads to run
    loop {
        while let Some(packet) = junction1.recv() {
            println!("Received packet: {:?}", packet.json);
        }
        junction1.dump_addresses();
        thread::sleep(Duration::from_secs(2)); // Add delay of 5 seconds
    }
}
